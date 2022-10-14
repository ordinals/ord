use super::*;

#[derive(Debug, Parser)]
#[clap(version)]
pub(crate) struct Arguments {
  #[clap(flatten)]
  pub(crate) options: Options,
  #[clap(subcommand)]
  pub(crate) subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    self.subcommand.run(self.options)
  }
}
