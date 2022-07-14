use {super::*, Wallet::*};

mod init;

#[derive(Parser)]
pub(crate) enum Wallet {
  Init,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Init => init::run(options),
    }
  }
}
