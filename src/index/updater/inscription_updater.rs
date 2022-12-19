use super::*;

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) id_to_satpoint: &'a mut Table<'db, 'tx, &'tx InscriptionIdArray, &'tx SatPointArray>,
  pub(super) next_number: &'a mut u64,
  pub(super) number_to_id: &'a mut Table<'db, 'tx, u64, &'tx InscriptionIdArray>,
  pub(super) satpoint_to_id: &'a mut Table<'db, 'tx, &'tx SatPointArray, &'tx InscriptionIdArray>,
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

      self.id_to_satpoint.insert(inscription_id, &satpoint)?;
      self.satpoint_to_id.insert(&satpoint, inscription_id)?;
      self.number_to_id.insert(self.next_number, inscription_id)?;
      *self.next_number += 1;
    };

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

      let inscription_ids: Vec<(SatPointArray, InscriptionIdArray)> = self
        .satpoint_to_id
        .range(start..=end)?
        .map(|(satpoint, id)| (*satpoint, *id))
        .collect();

      for (old_satpoint, inscription_id) in inscription_ids {
        let new_satpoint = encode_satpoint(SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        });

        self.satpoint_to_id.remove(&old_satpoint)?;
        self.satpoint_to_id.insert(&new_satpoint, &inscription_id)?;
        self.id_to_satpoint.insert(&inscription_id, &new_satpoint)?;
      }
    }

    Ok(inscribed)
  }
}
