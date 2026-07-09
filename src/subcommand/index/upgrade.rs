use super::*;

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  crate::index::migrations::run(&settings)?;

  Ok(None)
}
