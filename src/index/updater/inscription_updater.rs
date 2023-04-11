use {super::*, std::collections::BTreeSet};

pub(super) struct Flotsam {
  inscription_id: InscriptionId,
  offset: u64,
  origin: Origin,
}

enum Origin {
  New(u64),
  Old(SatPoint),
}

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  flotsam: Vec<Flotsam>,
  height: u64,
  id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
  value_receiver: &'a mut Receiver<u64>,
  id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  lost_sats: u64,
  next_number: u64,
  number_to_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
  outpoint_to_value: &'a mut Table<'db, 'tx, &'static OutPointValue, u64>,
  reward: u64,
  sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
  satpoint_to_id: &'a mut Table<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
  timestamp: u32,
  value_cache: &'a mut HashMap<OutPoint, u64>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    height: u64,
    id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
    value_receiver: &'a mut Receiver<u64>,
    id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
    lost_sats: u64,
    number_to_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
    outpoint_to_value: &'a mut Table<'db, 'tx, &'static OutPointValue, u64>,
    sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
    satpoint_to_id: &'a mut Table<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
    timestamp: u32,
    value_cache: &'a mut HashMap<OutPoint, u64>,
  ) -> Result<Self> {
    let next_number = number_to_id
      .iter()?
      .rev()
      .map(|(number, _id)| number.value() + 1)
      .next()
      .unwrap_or(0);

    Ok(Self {
      flotsam: Vec::new(),
      height,
      id_to_satpoint,
      value_receiver,
      id_to_entry,
      lost_sats,
      next_number,
      number_to_id,
      outpoint_to_value,
      reward: Height(height).subsidy(),
      sat_to_inscription_id,
      satpoint_to_id,
      timestamp,
      value_cache,
    })
  }

  pub(super) fn index_transaction_inscriptions(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
  ) -> Result<u64> {
    let mut floating_inscriptions = Vec::new();
    let mut inscribed_offsets = BTreeSet::new();
    let mut input_value = 0;
    let mut id_index = 0;
    for tx_in in &tx.input {
      // skip subsidy since no inscriptions possible
      if tx_in.previous_output.is_null() {
        input_value += Height(self.height).subsidy();
        continue;
      }

      // find existing inscriptions on input aka transfers of inscriptions
      for (old_satpoint, inscription_id) in
        Index::inscriptions_on_output(self.satpoint_to_id, tx_in.previous_output)?
      {
        floating_inscriptions.push(Flotsam {
          offset: input_value + old_satpoint.offset,
          inscription_id,
          origin: Origin::Old(old_satpoint),
        });

        inscribed_offsets.insert(input_value + old_satpoint.offset);
      }

      // find new inscriptions
      if let Some(_inscription) = Inscription::from_tx_input(tx_in) {
        // ignore new inscriptions on already inscribed offset (sats)
        if !inscribed_offsets.contains(&input_value) {
          let inscription_id = InscriptionId {
            txid,
            index: id_index,
          };

          id_index += 1;

          floating_inscriptions.push(Flotsam {
            inscription_id,
            offset: input_value,
            origin: Origin::New(0), // 0 is placeholder, calculate fee later
          });
        }
      }

      // different ways to get the utxo set (input amount)
      input_value += if let Some(value) = self.value_cache.remove(&tx_in.previous_output) {
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
      }
    }

    // calulate genesis fee for new inscriptions
    let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<u64>();
    let mut floating_inscriptions = floating_inscriptions
      .into_iter()
      .map(|flotsam| {
        if let Flotsam {
          inscription_id,
          offset,
          origin: Origin::New(_),
        } = flotsam
        {
          Flotsam {
            inscription_id,
            offset,
            origin: Origin::New(input_value - total_output_value),
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

        self.update_inscription_location(
          input_sat_ranges,
          inscriptions.next().unwrap(), // This will need to change when we implement multiple inscriptions per TX (#1298).
          new_satpoint,
        )?;
      }
      output_value = end;

      self.value_cache.insert(
        OutPoint {
          vout: vout.try_into().unwrap(),
          txid,
        },
        tx_out.value,
      );
    }

    if is_coinbase {
      for flotsam in inscriptions {
        let new_satpoint = SatPoint {
          outpoint: OutPoint::null(),
          offset: self.lost_sats + flotsam.offset - output_value,
        };
        self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
      }

      Ok(self.reward - output_value)
    } else {
      self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
        offset: self.reward + flotsam.offset,
        ..flotsam
      }));
      self.reward += input_value - output_value;
      Ok(0)
    }
  }

  fn update_inscription_location(
    &mut self,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    flotsam: Flotsam,
    new_satpoint: SatPoint,
  ) -> Result {
    let inscription_id = flotsam.inscription_id.store();

    match flotsam.origin {
      Origin::Old(old_satpoint) => {
        self.satpoint_to_id.remove(&old_satpoint.store())?;
      }
      Origin::New(fee) => {
        self
          .number_to_id
          .insert(&self.next_number, &inscription_id)?;

        let mut sat = None;
        if let Some(input_sat_ranges) = input_sat_ranges {
          let mut offset = 0;
          for (start, end) in input_sat_ranges {
            let size = end - start;
            if offset + size > flotsam.offset {
              let n = start + flotsam.offset - offset;
              self.sat_to_inscription_id.insert(&n, &inscription_id)?;
              sat = Some(Sat(n));
              break;
            }
            offset += size;
          }
        }

        self.id_to_entry.insert(
          &inscription_id,
          &InscriptionEntry {
            fee,
            height: self.height,
            number: self.next_number,
            sat,
            timestamp: self.timestamp,
          }
          .store(),
        )?;

        self.next_number += 1;
      }
    }

    let new_satpoint = new_satpoint.store();

    self.satpoint_to_id.insert(&new_satpoint, &inscription_id)?;
    self.id_to_satpoint.insert(&inscription_id, &new_satpoint)?;

    Ok(())
  }
}
