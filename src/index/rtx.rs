use super::*;

pub(crate) struct Rtx<'a>(pub(crate) redb::ReadTransaction<'a>);

impl Rtx<'_> {
  pub(crate) fn height(&self) -> Result<u64> {
    let height_to_hash = self.0.open_table(HEIGHT_TO_HASH)?;

    Ok(
      height_to_hash
        .range(0..)?
        .rev()
        .next()
        .map(|(height, _hash)| height)
        .unwrap_or(0),
    )
  }
}
