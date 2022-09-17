use super::*;

pub(crate) struct Rtx<'a>(pub(crate) redb::ReadTransaction<'a>);

impl Rtx<'_> {
  pub(crate) fn height(&self) -> Result<u64> {
    Index::read_height_from_table(self.0.open_table(HEIGHT_TO_HASH)?)
  }
}
