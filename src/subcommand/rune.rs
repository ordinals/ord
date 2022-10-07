use super::*;

mod publish;

#[derive(Debug, Parser)]
pub(crate) enum Rune {
  Publish(publish::Publish),
}

impl Rune {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Publish(publish) => publish.run(options),
    }
  }
}
