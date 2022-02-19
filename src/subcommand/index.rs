use super::*;

pub(crate) fn run(options: Options) -> Result<()> {
  Index::new(options)?;
  Ok(())
}
