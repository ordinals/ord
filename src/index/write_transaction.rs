use super::*;

pub(crate) struct WriteTransaction<'a> {
  pub(crate) inner: DatabaseTransaction<'a>,
  pub(crate) height_to_hash: Table<'a, u64, [u8]>,
  pub(crate) outpoint_to_ordinal_ranges: Table<'a, [u8], [u8]>,
}

impl<'a> WriteTransaction<'a> {
  pub(crate) fn new(database: &'a redb::Database) -> Result<Self> {
    let inner = database.begin_write()?;
    let height_to_hash = inner.open_table(&HEIGHT_TO_HASH)?;
    let outpoint_to_ordinal_ranges = inner.open_table(&OUTPOINT_TO_ORDINAL_RANGES)?;

    Ok(Self {
      inner,
      height_to_hash,
      outpoint_to_ordinal_ranges,
    })
  }

  pub(crate) fn height(&self) -> Result<u64> {
    Ok(
      self
        .height_to_hash
        .range(0..)?
        .rev()
        .next()
        .map(|(height, _hash)| height + 1)
        .unwrap_or(0),
    )
  }

  pub(crate) fn blockhash_at_height(&self, height: u64) -> Result<Option<&[u8]>> {
    Ok(self.height_to_hash.get(&height)?)
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    self.height_to_hash.insert(&height, &blockhash)?;
    Ok(())
  }

  pub(crate) fn remove_outpoint(&mut self, outpoint: &[u8]) -> Result {
    self.outpoint_to_ordinal_ranges.remove(outpoint)?;
    Ok(())
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<&[u8]>> {
    Ok(self.outpoint_to_ordinal_ranges.get(outpoint)?)
  }

  pub(crate) fn commit(self) -> Result {
    self.inner.commit()?;
    Ok(())
  }
}
