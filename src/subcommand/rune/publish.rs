use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  rune: crate::Rune,
}

impl Publish {
  pub(crate) fn run(self, _options: Options) -> Result {
    Ok(())
  }
}
