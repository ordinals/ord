use super::*;

#[derive(StructOpt)]
pub(crate) struct Arguments {
  #[structopt(flatten)]
  options: Options,
  #[structopt(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result<()> {
    self.subcommand.run(self.options)
  }
}
