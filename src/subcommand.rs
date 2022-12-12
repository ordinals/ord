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
  #[clap(about = "List the first satoshis of each reward epoch")]
  Epochs,
  #[clap(about = "Find a satoshi's current location")]
  Find(find::Find),
  #[clap(about = "Update the index")]
  Index,
  #[clap(about = "Display index statistics")]
  Info(info::Info),
  #[clap(about = "List the satoshis in an output")]
  List(list::List),
  #[clap(about = "Parse a satoshi from ordinal notation")]
  Parse(parse::Parse),
  #[clap(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[clap(about = "Run the explorer server")]
  Server(server::Server),
  #[clap(about = "Display Bitcoin supply information")]
  Supply,
  #[clap(about = "Display satoshi traits")]
  Traits(traits::Traits),
  #[clap(subcommand, about = "Wallet commands")]
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
