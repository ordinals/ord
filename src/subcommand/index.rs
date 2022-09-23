use super::*;

pub(crate) fn run(options: Options) -> Result<()> {
  let index = Index::open(&options)?;

  index.index()?;

  Ok(())
}
