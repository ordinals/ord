use super::*;

#[derive(StructOpt)]
pub(crate) struct Arguments {
  #[structopt(long)]
  index_size: Option<usize>,
  #[structopt(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result<()> {
    self.subcommand.run(self.index_size)
  }
}
