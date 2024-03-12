use super::*;

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let index = Index::open(&settings)?;

  index.update()?;

  Ok(None)
}
