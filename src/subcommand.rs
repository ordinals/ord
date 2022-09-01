use super::*;

mod epochs;
mod find;
mod index;
mod info;
mod list;
mod name;
mod range;
mod server;
mod supply;
mod traits;
mod wallet;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  Epochs,
  Find(find::Find),
  Index,
  Info,
  List(list::List),
  Name(name::Name),
  Range(range::Range),
  Server(server::Server),
  Supply,
  Traits(traits::Traits),
  #[clap(subcommand)]
  Wallet(wallet::Wallet),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(options),
      Self::Index => index::run(options),
      Self::Info => info::run(options),
      Self::List(list) => list.run(options),
      Self::Name(name) => name.run(),
      Self::Range(range) => range.run(),
      Self::Server(server) => server.run(options),
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
      Self::Wallet(wallet) => wallet.run(options),
    }
  }
}
