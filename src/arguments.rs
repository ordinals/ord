use super::*;

#[derive(StructOpt)]
pub(crate) struct Arguments {
  #[structopt(long)]
  index_size: Option<usize>,
  #[structopt(subcommand)]
  command: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result<()> {
    self.command.run(self.index_size)
  }
}
