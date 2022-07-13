use super::*;

mod init;

#[derive(Parser)]
pub(crate) enum Wallet {
  Init,
}

impl Wallet {
  pub(crate) fn run(self) -> Result {
    match self {
      Init => init::run(),
    }
  }
}
