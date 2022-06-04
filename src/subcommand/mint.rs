use super::*;

#[derive(Parser)]
pub(crate) struct Mint {
  #[clap(long)]
  ordinal: Ordinal,
}

impl Mint {
  pub(crate) fn run(self) -> Result<()> {
    Ok(())
  }
}
