use super::*;

mod balance;
mod fund;
mod identify;
mod init;
mod send;
mod utxos;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Balance,
  Fund,
  Identify,
  Init,
  Send(send::Send),
  Utxos,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Balance => balance::run(options),
      Self::Fund => fund::run(options),
      Self::Identify => identify::run(options),
      Self::Init => init::run(options),
      Self::Send(send) => send.run(options),
      Self::Utxos => utxos::run(options),
    }
  }
}
