use super::*;

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) height: u64,
  pub(super) id_to_height: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, u64>,
  pub(super) id_to_satpoint: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, &'tx SatPointArray>,
  pub(super) next_number: &'a mut u64,
  pub(super) number_to_id: &'a mut Table<'db, 'tx, u64, &'tx InscriptionIdArray>,
  pub(super) satpoint_to_id: &'a mut Table<'db, 'tx, &'tx SatPointArray, &'tx InscriptionIdArray>,
  pub(super) outpoint_to_value: &'a mut Table<'db, 'tx, &'tx OutPointArray, u64>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn index_transaction_inscriptions(
    &mut self,
    tx: &Transaction,
    txid: Txid,
  ) -> Result<bool> {
    let inscribed = Inscription::from_transaction(tx).is_some();

    if inscribed {
      let satpoint = encode_satpoint(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      });

      let inscription_id = txid.as_inner();

      self.id_to_height.insert(inscription_id, &self.height)?;
      self.id_to_satpoint.insert(inscription_id, &satpoint)?;
      self.satpoint_to_id.insert(&satpoint, inscription_id)?;
      self.number_to_id.insert(self.next_number, inscription_id)?;
      *self.next_number += 1;
    };

    let mut inscriptions: Vec<(u64, InscriptionIdArray, SatPointArray)> = Vec::new();

    let mut offset = 0;
    for tx_in in &tx.input {
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
        inscriptions.push((
          offset + decode_satpoint(old_satpoint).offset,
          inscription_id,
          old_satpoint,
        ));
      }

      // Options:
      // - remove transaction skipping optimization
      // - get transactions ad hoc from core if we have missing values

      if !tx_in.previous_output.is_null() {
        offset += self
          .outpoint_to_value
          .get(&encode_outpoint(tx_in.previous_output))?
          .unwrap()
          .value();
      }

      self
        .outpoint_to_value
        .remove(&encode_outpoint(tx_in.previous_output))?;
    }

    inscriptions.sort();
    let mut inscriptions = inscriptions.into_iter().peekable();

    let mut start = 0;
    for (vout, tx_out) in tx.output.iter().enumerate() {
      let end = start + tx_out.value;

      while let Some((offset, inscription_id, old_satpoint)) = inscriptions.peek() {
        if *offset >= end {
          break;
        }

        let new_satpoint = encode_satpoint(SatPoint {
          outpoint: OutPoint {
            txid,
            vout: vout.try_into().unwrap(),
          },
          offset: offset - start,
        });

        // TODO: test that we're removing the old satpoint
        self.satpoint_to_id.remove(&old_satpoint)?;
        self.satpoint_to_id.insert(&new_satpoint, &inscription_id)?;
        self.id_to_satpoint.insert(&inscription_id, &new_satpoint)?;

        inscriptions.next();
      }

      start = end;
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

    if inscriptions.next().is_some() {
      todo!("handle inscription being lost to fee");
    }

    Ok(inscribed)
  }
}
