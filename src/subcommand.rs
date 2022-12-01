use super::*;

mod epochs;
mod find;
mod index;
mod info;
mod list;
mod parse;
mod server;
mod subsidy;
mod supply;
mod traits;
mod wallet;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  Epochs,
  Find(find::Find),
  Index,
  Info(info::Info),
  List(list::List),
  Parse(parse::Parse),
  Subsidy(subsidy::Subsidy),
  Server(server::Server),
  Supply,
  Traits(traits::Traits),
  #[clap(subcommand)]
  Wallet(wallet::Wallet),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(options),
      Self::Index => index::run(options),
      Self::Info(info) => info.run(options),
      Self::List(list) => list.run(options),
      Self::Parse(parse) => parse.run(),
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(options, index, handle)
      }
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
      Self::Wallet(wallet) => wallet.run(options),
    }
  }
}
