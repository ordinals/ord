use super::*;

pub(crate) fn run(options: Options) -> Result<()> {
  Index::index(options)?;
  Ok(())
}
