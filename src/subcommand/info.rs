use super::*;

pub(crate) fn run(options: Options) -> Result {
  Index::open(&options)?.print_info()?;
  Ok(())
}
