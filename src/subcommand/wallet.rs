use super::*;

mod fund;
mod init;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Init,
  Fund,
}

impl Wallet {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Init => init::run(),
      Self::Fund => fund::run(),
    }
  }
}
