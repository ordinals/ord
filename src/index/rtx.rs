use super::*;
const PAGE_SIZE: u32 = 100;
pub struct Rtx(pub redb::ReadTransaction);

impl Rtx {
  pub(crate) fn block_height(&self) -> Result<Option<Height>> {
    Ok(
      self
        .0
        .open_table(HEIGHT_TO_BLOCK_HEADER)?
        .range(0..)?
        .next_back()
        .transpose()?
        .map(|(height, _header)| Height(height.value())),
    )
  }

  pub(crate) fn block_count(&self) -> Result<u32> {
    Ok(
      self
        .0
        .open_table(HEIGHT_TO_BLOCK_HEADER)?
        .range(0..)?
        .next_back()
        .transpose()?
        .map(|(height, _header)| height.value() + 1)
        .unwrap_or(0),
    )
  }

  pub(crate) fn block_hash(&self, height: Option<u32>) -> Result<Option<BlockHash>> {
    let height_to_block_header = self.0.open_table(HEIGHT_TO_BLOCK_HEADER)?;

    Ok(
      match height {
        Some(height) => height_to_block_header.get(height)?,
        None => height_to_block_header
          .range(0..)?
          .next_back()
          .transpose()?
          .map(|(_height, header)| header),
      }
      .map(|header| Header::load(*header.value()).block_hash()),
    )
  }
  pub fn block_hashes_in_interval_paginated(
    &self,
    start: u32,
    interval: u32,
    page: u32,
  ) -> Result<(Vec<bitcoin::BlockHash>, bool)> {
    let height_to_block_header = self.0.open_table(HEIGHT_TO_BLOCK_HEADER)?;
    let mut block_hashes = Vec::new();

    let mut current_height =
      start.saturating_add(page.saturating_mul(PAGE_SIZE).saturating_mul(interval));

    let mut fetched = 0;
    while fetched < PAGE_SIZE {
      if let Some(header) = height_to_block_header.get(current_height)? {
        let block_hash = Header::load(*header.value()).block_hash();
        block_hashes.push(block_hash);
      }
      current_height = current_height.saturating_add(interval);
      fetched += 1;
    }

    let more_blocks = fetched == PAGE_SIZE;
    Ok((block_hashes, more_blocks))
  }
}
