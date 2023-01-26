use super::*;

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  index.update()?;

  Ok(Box::new(Empty {}))
}
