use super::*;

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  Ok(Some(Box::new(settings)))
}
