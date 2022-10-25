use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;
  index.print_info()?;
  Ok(())
}
