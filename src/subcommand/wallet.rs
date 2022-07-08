use super::*;

#[derive(Parser)]
pub(crate) struct Wallet {
  #[clap(long)]
  init: bool
}

impl Wallet {
  pub(crate) fn run(self) -> Result {
    if self.init {
      println!("wallet initialized!");
    }

    Ok(())
  }
}
