use super::*;

pub(crate) fn run(index_size: Option<usize>) -> Result<()> {
  Index::new(index_size)?;
  Ok(())
}
