use super::*;

#[derive(Default)]
pub struct Cache {
  outpoint_to_ordinal_ranges_map: HashMap<[u8; 36], Vec<u8>>,
  pub(crate) outputs_traversed: u64,
  outputs_cached: u64,
  outputs_inserted_since_flush: u64,
}

impl Cache {
  fn flush(&mut self, wtx: &mut WriteTransaction) -> Result {
    log::info!(
      "Flushing {} entries ({:.1}% resulting from {} insertions) from memory to database",
      self.outpoint_to_ordinal_ranges_map.len(),
      self.outpoint_to_ordinal_ranges_map.len() as f64 / self.outputs_inserted_since_flush as f64
        * 100.,
      self.outputs_inserted_since_flush,
    );
    let mut outpoint_to_ordinal_ranges = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

    for (k, v) in &self.outpoint_to_ordinal_ranges_map {
      outpoint_to_ordinal_ranges.insert(k, v)?;
    }

    self.outpoint_to_ordinal_ranges_map.clear();
    self.outputs_inserted_since_flush = 0;
    Ok(())
  }

  pub(crate) fn get_and_remove(
    &mut self,
    outpoint: OutPoint,
    outpoint_to_ordinal_ranges: &mut Table<[u8; 36], [u8]>,
  ) -> Result<Vec<u8>> {
    let key = encode_outpoint(outpoint);
    match self.outpoint_to_ordinal_ranges_map.remove(&key) {
      Some(ord_range_vec) => {
        self.outputs_cached += 1;
        Ok(ord_range_vec)
      }
      None => {
        let ord_range = outpoint_to_ordinal_ranges
          .remove(&key)?
          .ok_or_else(|| anyhow!("Could not find outpoint {} in index", outpoint))?;
        Ok(ord_range.to_value().to_vec())
      }
    }
  }

  pub(crate) fn insert(&mut self, outpoint: &mut OutPoint, ordinals: Vec<u8>) {
    let key = encode_outpoint(*outpoint);
    self.outpoint_to_ordinal_ranges_map.insert(key, ordinals);
    self.outputs_inserted_since_flush += 1;
  }

  pub(crate) fn commit(&mut self, mut wtx: WriteTransaction, height: u64) -> Result {
    log::info!(
      "Committing at block height {}, {} outputs traversed, {} in map, {} cached",
      height,
      self.outputs_traversed,
      self.outpoint_to_ordinal_ranges_map.len(),
      self.outputs_cached
    );

    self.flush(&mut wtx)?;

    Index::increment_statistic(&wtx, Statistic::OutputsTraversed, self.outputs_traversed)?;
    Index::increment_statistic(&wtx, Statistic::Commits, 1)?;
    wtx.commit()?;
    Ok(())
  }
}
