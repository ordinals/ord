use super::*;

mod fund;
mod init;

pub(crate) fn get_key() -> Result<impl DerivableKey<Segwitv0> + Clone> {
  Ok((
    Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")?,
    None,
  ))
}

#[derive(Parser)]
pub(crate) enum Wallet {
  Init,
  Fund,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Init => init::run(options),
      Self::Fund => fund::run(),
    }
  }
}
