use super::*;

#[derive(Clone)]
pub struct BlockIndex {
  first_inscription_height: u64,
  lowest_blessed_by_block: Vec<i64>,
  lowest_cursed_by_block: Vec<i64>,
  highest_indexed_blessed: i64,
  lowest_indexed_cursed: i64,
}

impl BlockIndex {
  pub(crate) fn new(index: &Index) -> Result<BlockIndex> {
    Ok(BlockIndex {
      first_inscription_height: index.options.first_inscription_height(),
      lowest_blessed_by_block: Vec::new(),
      lowest_cursed_by_block: Vec::new(),
      highest_indexed_blessed: i64::MIN,
      lowest_indexed_cursed: i64::MAX,
    })
  }

  pub(crate) fn update(&mut self, index: &Index) -> Result {
    let index_height = index.block_count()?;
    let inscribed_block_count = index_height.saturating_sub(self.first_inscription_height);
    let indexed_up_to: isize = self
      .lowest_blessed_by_block
      .len()
      .try_into()
      .unwrap_or(isize::MAX);

    let gap = inscribed_block_count.try_into().unwrap_or(isize::MAX) - indexed_up_to;

    log::info!(
      "Indexed up to {} ({} inscribed blocks, {} indexed blocks, {} gap, {} lowest indexed cursed, {} highest indexed blessed)",
      indexed_up_to,
      inscribed_block_count,
      indexed_up_to,
      gap,
      self.lowest_indexed_cursed,
      self.highest_indexed_blessed,
    );

    if gap > 0 {
      log::info!(
        "Updating block index for {} new blocks ({} to {})",
        gap,
        indexed_up_to,
        inscribed_block_count
      );
      self
        .lowest_blessed_by_block
        .resize(usize::try_from(inscribed_block_count)?, i64::MAX);
      self
        .lowest_cursed_by_block
        .resize(usize::try_from(inscribed_block_count)?, i64::MAX);

      let mut prev_block_height = usize::MAX;

      let rtx = index.database.begin_read()?;
      for result in rtx
        .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
        .iter()?
      {
        let (number, id) = result?;
        if number.value() >= self.lowest_indexed_cursed
          && number.value() <= self.highest_indexed_blessed
        {
          continue;
        }

        let inscription_id = InscriptionId::load(*id.value());
        if let Some(entry) = index.get_inscription_entry(inscription_id)? {
          let current_height = entry.height.try_into().unwrap_or(usize::MAX);
          if current_height % 5000 == 0 {
            log::debug!("Updating block index at {}", current_height);
          }
          if prev_block_height != current_height {
            prev_block_height = current_height;
            if number.value() < 0 {
              self.lowest_cursed_by_block[prev_block_height
                .saturating_sub(usize::try_from(self.first_inscription_height)?)] = number.value();
              self.lowest_indexed_cursed = cmp::min(self.lowest_indexed_cursed, number.value());
            } else {
              self.lowest_blessed_by_block[prev_block_height
                .saturating_sub(usize::try_from(self.first_inscription_height)?)] = number.value();
              self.highest_indexed_blessed = cmp::max(self.highest_indexed_blessed, number.value());
            }
          }
        }
      }
      log::info!(
        "Updated block index for {} new blocks ({} to {})",
        gap,
        indexed_up_to,
        inscribed_block_count
      );
    }

    Ok(())
  }

  // Return all consecutively numbered inscriptions in the block at the given height, starting from the given number
  fn get_inscriptions_in_block_from(
    &self,
    index: &Index,
    block_height: u64,
    from_number: i64,
  ) -> Result<Vec<InscriptionId>> {
    let mut block_inscriptions = Vec::new();

    let rtx = index.database.begin_read()?;
    let inscription_id_by_number = rtx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;

    let highest = match inscription_id_by_number.iter()?.next_back() {
      Some(Ok((number, _id))) => number.value(),
      Some(Err(err)) => return Err(err.into()),
      None => i64::MIN,
    };

    for number in from_number..highest {
      match inscription_id_by_number.get(number)? {
        Some(inscription_id) => {
          let inscription_id = InscriptionId::load(*inscription_id.value());
          if let Some(entry) = index.get_inscription_entry(inscription_id)? {
            if entry.height != block_height {
              break;
            }
            block_inscriptions.push(inscription_id);
          }
        }
        None => break,
      }
    }

    Ok(block_inscriptions)
  }

  pub(crate) fn get_inscriptions_in_block(
    &self,
    index: &Index,
    block_height: u64,
  ) -> Result<Vec<InscriptionId>> {
    if block_height >= index.block_count()? || block_height < self.first_inscription_height {
      return Ok(Vec::new());
    }
    let lowest_cursed = self.lowest_cursed_by_block
      [usize::try_from(block_height.saturating_sub(self.first_inscription_height))?];
    let lowest_blessed = self.lowest_blessed_by_block
      [usize::try_from(block_height.saturating_sub(self.first_inscription_height))?];

    log::info!(
      "Getting inscriptions in block {} ({} - {})",
      block_height,
      lowest_cursed,
      lowest_blessed
    );

    let mut inscriptions =
      self.get_inscriptions_in_block_from(index, block_height, lowest_cursed)?;
    inscriptions.extend(self.get_inscriptions_in_block_from(
      index,
      block_height,
      lowest_blessed,
    )?);
    log::info!(
      "Got {} inscriptions in block {}",
      inscriptions.len(),
      block_height
    );

    Ok(inscriptions)
  }
}
