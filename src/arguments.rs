use super::*;

#[derive(Parser)]
#[clap(version)]
pub(crate) struct Arguments {
  #[clap(flatten)]
  options: Options,
  #[clap(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result<()> {
    self.subcommand.run(self.options)
  }
}
