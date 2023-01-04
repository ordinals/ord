use super::*;

pub(super) struct Flotsam {
  inscription_id: InscriptionId,
  offset: u64,
  origin: Origin,
}

enum Origin {
  New,
  Old(SatPoint),
}

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  flotsam: Vec<Flotsam>,
  height: u64,
  id_to_height: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, u64>,
  id_to_satpoint: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, &'tx SatPointArray>,
  index: &'a Index,
  lost_sats: u64,
  next_number: u64,
  number_to_id: &'a mut Table<'db, 'tx, u64, &'tx InscriptionIdArray>,
  outpoint_to_value: &'a mut Table<'db, 'tx, &'tx OutPointArray, u64>,
  reward: u64,
  sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'tx InscriptionIdArray>,
  satpoint_to_id: &'a mut Table<'db, 'tx, &'tx SatPointArray, &'tx InscriptionIdArray>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    height: u64,
    id_to_height: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, u64>,
    id_to_satpoint: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, &'tx SatPointArray>,
    index: &'a Index,
    lost_sats: u64,
    number_to_id: &'a mut Table<'db, 'tx, u64, &'tx InscriptionIdArray>,
    outpoint_to_value: &'a mut Table<'db, 'tx, &'tx OutPointArray, u64>,
    sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'tx InscriptionIdArray>,
    satpoint_to_id: &'a mut Table<'db, 'tx, &'tx SatPointArray, &'tx InscriptionIdArray>,
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
      id_to_height,
      id_to_satpoint,
      index,
      lost_sats,
      next_number,
      number_to_id,
      outpoint_to_value,
      reward: Height(height).subsidy(),
      sat_to_inscription_id,
      satpoint_to_id,
    })
  }

  pub(super) fn index_transaction_inscriptions(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
  ) -> Result<u64> {
    let mut inscriptions = Vec::new();

    if Inscription::from_transaction(tx).is_some() {
      inscriptions.push(Flotsam {
        inscription_id: txid,
        offset: 0,
        origin: Origin::New,
      });
    };

    let mut input_value = 0;
    for tx_in in &tx.input {
      if tx_in.previous_output.is_null() {
        input_value += Height(self.height).subsidy();
      } else {
        let outpoint = tx_in.previous_output;
        let start = encode_satpoint(SatPoint {
          outpoint,
          offset: 0,
        });

        let end = encode_satpoint(SatPoint {
          outpoint,
          offset: u64::MAX,
        });

        for (old_satpoint, inscription_id) in self
          .satpoint_to_id
          .range(start..=end)?
          .map(|(satpoint, id)| (*satpoint.value(), *id.value()))
        {
          let old_satpoint = decode_satpoint(old_satpoint);
          inscriptions.push(Flotsam {
            offset: input_value + old_satpoint.offset,
            inscription_id: InscriptionId::from_inner(inscription_id),
            origin: Origin::Old(old_satpoint),
          });
        }
        self
          .outpoint_to_value
          .remove(&encode_outpoint(tx_in.previous_output))?;

        input_value += if let Some(value) = self
          .outpoint_to_value
          .get(&encode_outpoint(tx_in.previous_output))?
        {
          value.value()
        } else {
          self
            .index
            .get_transaction(tx_in.previous_output.txid)?
            .ok_or_else(|| {
              anyhow!(
                "failed to get transaction for {}",
                tx_in.previous_output.txid
              )
            })?
            .output[usize::try_from(tx_in.previous_output.vout).unwrap()]
          .value
        }
      }
    }

    let is_coinbase = tx
      .input
      .first()
      .map(|tx_in| tx_in.previous_output.is_null())
      .unwrap_or_default();

    if is_coinbase {
      inscriptions.append(&mut self.flotsam);
    }

    inscriptions.sort_by_key(|flotsam| flotsam.offset);
    let mut inscriptions = inscriptions.into_iter().peekable();

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
          inscriptions.next().unwrap(),
          new_satpoint,
        )?;
      }

      output_value = end;
    }

    for (vout, tx_out) in tx.output.iter().enumerate() {
      self.outpoint_to_value.insert(
        &encode_outpoint(OutPoint {
          vout: vout.try_into().unwrap(),
          txid,
        }),
        &tx_out.value,
      )?;
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
    let inscription_id = flotsam.inscription_id.into_inner();

    match flotsam.origin {
      Origin::Old(old_satpoint) => {
        self.satpoint_to_id.remove(&encode_satpoint(old_satpoint))?;
      }
      Origin::New => {
        self.id_to_height.insert(&inscription_id, &self.height)?;
        self
          .number_to_id
          .insert(&self.next_number, &inscription_id)?;

        if let Some(input_sat_ranges) = input_sat_ranges {
          let mut offset = 0;
          for (start, end) in input_sat_ranges {
            let size = end - start;
            if offset + size > flotsam.offset {
              self
                .sat_to_inscription_id
                .insert(&(start + flotsam.offset - offset), &inscription_id)?;
              break;
            }
            offset += size;
          }
        }

        self.next_number += 1;
      }
    }

    let new_satpoint = encode_satpoint(new_satpoint);

    self.satpoint_to_id.insert(&new_satpoint, &inscription_id)?;
    self.id_to_satpoint.insert(&inscription_id, &new_satpoint)?;

    Ok(())
  }
}
