use {
  super::*,
  bitcoincore_rpc::{Auth, Client, RpcApi},
};

mod identify;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Identify,
  List,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Identify => identify::run(options),
      Self::List => Ok(()),
    }
  }
}

#[cfg(test)]
mod tests {}
