use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;
  serde_json::to_writer(io::stdout(), &index.info()?)?;
  Ok(())
}
