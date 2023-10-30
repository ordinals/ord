use {super::*, inscription::Curse};

#[derive(Debug, Clone)]
pub(super) struct Flotsam {
  inscription_id: InscriptionId,
  offset: u64,
  origin: Origin,
}

#[derive(Debug, Clone)]
enum Origin {
  New {
    cursed: bool,
    fee: u64,
    parent: Option<InscriptionId>,
    pointer: Option<u64>,
    unbound: bool,
  },
  Old {
    old_satpoint: SatPoint,
  },
}

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  flotsam: Vec<Flotsam>,
  height: u64,
  id_to_children:
    &'a mut MultimapTable<'db, 'tx, &'static InscriptionIdValue, &'static InscriptionIdValue>,
  id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
  value_receiver: &'a mut Receiver<u64>,
  id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  pub(super) lost_sats: u64,
  pub(super) cursed_inscription_count: u64,
  pub(super) blessed_inscription_count: u64,
  pub(super) next_sequence_number: u64,
  inscription_number_to_id: &'a mut Table<'db, 'tx, i64, &'static InscriptionIdValue>,
  sequence_number_to_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
  outpoint_to_value: &'a mut Table<'db, 'tx, &'static OutPointValue, u64>,
  reward: u64,
  sat_to_inscription_id: &'a mut MultimapTable<'db, 'tx, u64, &'static InscriptionIdValue>,
  satpoint_to_id:
    &'a mut MultimapTable<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
  timestamp: u32,
  pub(super) unbound_inscriptions: u64,
  value_cache: &'a mut HashMap<OutPoint, u64>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    height: u64,
    id_to_children: &'a mut MultimapTable<
      'db,
      'tx,
      &'static InscriptionIdValue,
      &'static InscriptionIdValue,
    >,
    id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
    value_receiver: &'a mut Receiver<u64>,
    id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
    lost_sats: u64,
    inscription_number_to_id: &'a mut Table<'db, 'tx, i64, &'static InscriptionIdValue>,
    cursed_inscription_count: u64,
    blessed_inscription_count: u64,
    sequence_number_to_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
    outpoint_to_value: &'a mut Table<'db, 'tx, &'static OutPointValue, u64>,
    sat_to_inscription_id: &'a mut MultimapTable<'db, 'tx, u64, &'static InscriptionIdValue>,
    satpoint_to_id: &'a mut MultimapTable<
      'db,
      'tx,
      &'static SatPointValue,
      &'static InscriptionIdValue,
    >,
    timestamp: u32,
    unbound_inscriptions: u64,
    value_cache: &'a mut HashMap<OutPoint, u64>,
  ) -> Result<Self> {
    let next_sequence_number = sequence_number_to_id
      .iter()?
      .next_back()
      .and_then(|result| result.ok())
      .map(|(number, _id)| number.value() + 1)
      .unwrap_or(0);

    Ok(Self {
      flotsam: Vec::new(),
      height,
      id_to_children,
      id_to_satpoint,
      value_receiver,
      id_to_entry,
      lost_sats,
      cursed_inscription_count,
      blessed_inscription_count,
      next_sequence_number,
      sequence_number_to_id,
      inscription_number_to_id,
      outpoint_to_value,
      reward: Height(height).subsidy(),
      sat_to_inscription_id,
      satpoint_to_id,
      timestamp,
      unbound_inscriptions,
      value_cache,
    })
  }

  pub(super) fn index_envelopes(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
  ) -> Result {
    let mut envelopes = ParsedEnvelope::from_transaction(tx).into_iter().peekable();
    let mut floating_inscriptions = Vec::new();
    let mut inscribed_offsets = BTreeMap::new();
    let mut total_input_value = 0;
    let mut id_counter = 0;

    for (input_index, tx_in) in tx.input.iter().enumerate() {
      // skip subsidy since no inscriptions possible
      if tx_in.previous_output.is_null() {
        total_input_value += Height(self.height).subsidy();
        continue;
      }

      // find existing inscriptions on input (transfers of inscriptions)
      for (old_satpoint, inscription_id) in Index::inscriptions_on_output_ordered(
        self.id_to_entry,
        self.satpoint_to_id,
        tx_in.previous_output,
      )? {
        let offset = total_input_value + old_satpoint.offset;
        floating_inscriptions.push(Flotsam {
          offset,
          inscription_id,
          origin: Origin::Old { old_satpoint },
        });

        inscribed_offsets
          .entry(offset)
          .and_modify(|(_id, count)| *count += 1)
          .or_insert((inscription_id, 0));
      }

      let offset = total_input_value;

      // multi-level cache for UTXO set to get to the input amount
      let current_input_value = if let Some(value) = self.value_cache.remove(&tx_in.previous_output)
      {
        value
      } else if let Some(value) = self
        .outpoint_to_value
        .remove(&tx_in.previous_output.store())?
      {
        value.value()
      } else {
        self.value_receiver.blocking_recv().ok_or_else(|| {
          anyhow!(
            "failed to get transaction for {}",
            tx_in.previous_output.txid
          )
        })?
      };

      total_input_value += current_input_value;

      // go through all inscriptions in this input
      while let Some(inscription) = envelopes.peek() {
        if inscription.input != u32::try_from(input_index).unwrap() {
          break;
        }

        let inscription_id = InscriptionId {
          txid,
          index: id_counter,
        };

        let curse = if inscription.payload.unrecognized_even_field {
          Some(Curse::UnrecognizedEvenField)
        } else if inscription.payload.duplicate_field {
          Some(Curse::DuplicateField)
        } else if inscription.payload.incomplete_field {
          Some(Curse::IncompleteField)
        } else if inscription.input != 0 {
          Some(Curse::NotInFirstInput)
        } else if inscription.offset != 0 {
          Some(Curse::NotAtOffsetZero)
        } else if inscription.payload.pointer.is_some() {
          Some(Curse::Pointer)
        } else if inscription.pushnum {
          Some(Curse::Pushnum)
        } else if inscribed_offsets.contains_key(&offset) {
          let seq_num = self.next_sequence_number;

          let sat = Self::calculate_sat(input_sat_ranges, offset);

          log::info!("processing reinscription {inscription_id} on sat {:?}: sequence number {seq_num}, inscribed offsets {:?}", sat, inscribed_offsets);

          Some(Curse::Reinscription)
        } else {
          None
        };

        if curse.is_some() {
          log::info!("found cursed inscription {inscription_id}: {:?}", curse);
        }

        let cursed = if let Some(Curse::Reinscription) = curse {
          let first_reinscription = inscribed_offsets
            .get(&offset)
            .map(|(_id, count)| count == &0)
            .unwrap_or(false);

          let initial_inscription_is_cursed = inscribed_offsets
            .get(&offset)
            .and_then(|(inscription_id, _count)| {
              match self.id_to_entry.get(&inscription_id.store()) {
                Ok(option) => option.map(|entry| {
                  let loaded_entry = InscriptionEntry::load(entry.value());
                  loaded_entry.inscription_number < 0
                }),
                Err(_) => None,
              }
            })
            .unwrap_or(false);

          log::info!("{inscription_id}: is first reinscription: {first_reinscription}, initial inscription is cursed: {initial_inscription_is_cursed}");

          !(initial_inscription_is_cursed && first_reinscription)
        } else {
          curse.is_some()
        };

        let unbound = current_input_value == 0 || curse == Some(Curse::UnrecognizedEvenField);

        if curse.is_some() || unbound {
          log::info!(
            "indexing inscription {inscription_id} with curse {:?} as cursed {} and unbound {}",
            curse,
            cursed,
            unbound
          );
        }

        floating_inscriptions.push(Flotsam {
          inscription_id,
          offset,
          origin: Origin::New {
            cursed,
            fee: 0,
            parent: inscription.payload.parent(),
            pointer: inscription.payload.pointer(),
            unbound,
          },
        });

        envelopes.next();
        id_counter += 1;
      }
    }

    let potential_parents = floating_inscriptions
      .iter()
      .map(|flotsam| flotsam.inscription_id)
      .collect::<HashSet<InscriptionId>>();

    for flotsam in &mut floating_inscriptions {
      if let Flotsam {
        origin: Origin::New { parent, .. },
        ..
      } = flotsam
      {
        if let Some(purported_parent) = parent {
          if !potential_parents.contains(purported_parent) {
            *parent = None;
          }
        }
      }
    }

    // still have to normalize over inscription size
    let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<u64>();
    let mut floating_inscriptions = floating_inscriptions
      .into_iter()
      .map(|flotsam| {
        if let Flotsam {
          inscription_id,
          offset,
          origin:
            Origin::New {
              cursed,
              fee: _,
              parent,
              pointer,
              unbound,
            },
        } = flotsam
        {
          Flotsam {
            inscription_id,
            offset,
            origin: Origin::New {
              fee: (total_input_value - total_output_value) / u64::from(id_counter),
              cursed,
              parent,
              pointer,
              unbound,
            },
          }
        } else {
          flotsam
        }
      })
      .collect::<Vec<Flotsam>>();

    let is_coinbase = tx
      .input
      .first()
      .map(|tx_in| tx_in.previous_output.is_null())
      .unwrap_or_default();

    if is_coinbase {
      floating_inscriptions.append(&mut self.flotsam);
    }

    floating_inscriptions.sort_by_key(|flotsam| flotsam.offset);
    let mut inscriptions = floating_inscriptions.into_iter().peekable();

    let mut range_to_vout = BTreeMap::new();
    let mut new_locations = Vec::new();
    let mut output_value = 0;
    for (vout, tx_out) in tx.output.iter().enumerate() {
      let end = output_value + tx_out.value;

      while let Some(flotsam) = inscriptions.peek() {
        if flotsam.offset >= end {
          break;
        }

        let new_satpoint = SatPoint {
          outpoint: OutPoint {
            txid,
            vout: vout.try_into().unwrap(),
          },
          offset: flotsam.offset - output_value,
        };

        new_locations.push((new_satpoint, inscriptions.next().unwrap()));
      }

      range_to_vout.insert((output_value, end), vout.try_into().unwrap());

      output_value = end;

      self.value_cache.insert(
        OutPoint {
          vout: vout.try_into().unwrap(),
          txid,
        },
        tx_out.value,
      );
    }

    for (new_satpoint, mut flotsam) in new_locations.into_iter() {
      let new_satpoint = match flotsam.origin {
        Origin::New {
          pointer: Some(pointer),
          ..
        } if pointer < output_value => {
          match range_to_vout.iter().find_map(|((start, end), vout)| {
            (pointer >= *start && pointer < *end).then(|| (vout, pointer - start))
          }) {
            Some((vout, offset)) => {
              flotsam.offset = pointer;
              SatPoint {
                outpoint: OutPoint { txid, vout: *vout },
                offset,
              }
            }
            _ => new_satpoint,
          }
        }
        _ => new_satpoint,
      };

      self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
    }

    if is_coinbase {
      for flotsam in inscriptions {
        let new_satpoint = SatPoint {
          outpoint: OutPoint::null(),
          offset: self.lost_sats + flotsam.offset - output_value,
        };
        self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
      }
      self.lost_sats += self.reward - output_value;
      Ok(())
    } else {
      self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
        offset: self.reward + flotsam.offset - output_value,
        ..flotsam
      }));
      self.reward += total_input_value - output_value;
      Ok(())
    }
  }

  fn calculate_sat(
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    input_offset: u64,
  ) -> Option<Sat> {
    let mut sat = None;
    if let Some(input_sat_ranges) = input_sat_ranges {
      let mut offset = 0;
      for (start, end) in input_sat_ranges {
        let size = end - start;
        if offset + size > input_offset {
          let n = start + input_offset - offset;
          sat = Some(Sat(n));
          break;
        }
        offset += size;
      }
    }
    sat
  }

  fn update_inscription_location(
    &mut self,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    flotsam: Flotsam,
    new_satpoint: SatPoint,
  ) -> Result {
    let inscription_id = flotsam.inscription_id.store();
    let unbound = match flotsam.origin {
      Origin::Old { old_satpoint } => {
        self.satpoint_to_id.remove_all(&old_satpoint.store())?;

        false
      }
      Origin::New {
        cursed,
        fee,
        parent,
        unbound,
        ..
      } => {
        let inscription_number = if cursed {
          let number: i64 = self.cursed_inscription_count.try_into().unwrap();
          self.cursed_inscription_count += 1;

          // because cursed numbers start at -1
          -(number + 1)
        } else {
          let number: i64 = self.blessed_inscription_count.try_into().unwrap();
          self.blessed_inscription_count += 1;

          number
        };

        self
          .inscription_number_to_id
          .insert(inscription_number, &inscription_id)?;

        let sequence_number = self.next_sequence_number;
        self.next_sequence_number += 1;

        self
          .sequence_number_to_id
          .insert(sequence_number, &inscription_id)?;

        let sat = if unbound {
          None
        } else {
          Self::calculate_sat(input_sat_ranges, flotsam.offset)
        };

        if let Some(Sat(n)) = sat {
          self.sat_to_inscription_id.insert(&n, &inscription_id)?;
        }

        self.id_to_entry.insert(
          &inscription_id,
          &InscriptionEntry {
            fee,
            height: self.height,
            inscription_number,
            sequence_number,
            parent,
            sat,
            timestamp: self.timestamp,
          }
          .store(),
        )?;

        if let Some(parent) = parent {
          self
            .id_to_children
            .insert(&parent.store(), &inscription_id)?;
        }

        unbound
      }
    };

    let satpoint = if unbound {
      let new_unbound_satpoint = SatPoint {
        outpoint: unbound_outpoint(),
        offset: self.unbound_inscriptions,
      };
      self.unbound_inscriptions += 1;
      new_unbound_satpoint.store()
    } else {
      new_satpoint.store()
    };

    self.satpoint_to_id.insert(&satpoint, &inscription_id)?;
    self.id_to_satpoint.insert(&inscription_id, &satpoint)?;

    Ok(())
  }
}
