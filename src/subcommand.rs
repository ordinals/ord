use super::*;

pub mod decode;
pub mod epochs;
pub mod find;
pub mod index;
pub mod list;
pub mod parse;
mod preview;
mod server;
pub mod subsidy;
pub mod supply;
pub mod teleburn;
pub mod traits;
pub mod wallet;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "Decode a transaction")]
  Decode(decode::Decode),
  #[command(about = "List the first satoshis of each reward epoch")]
  Epochs,
  #[command(about = "Find a satoshi's current location")]
  Find(find::Find),
  #[command(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[command(about = "List the satoshis in an output")]
  List(list::List),
  #[command(about = "Parse a satoshi from ordinal notation")]
  Parse(parse::Parse),
  #[command(about = "Run an explorer server populated with inscriptions")]
  Preview(preview::Preview),
  #[command(about = "Run the explorer server")]
  Server(server::Server),
  #[command(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[command(about = "Display Bitcoin supply information")]
  Supply,
  #[command(about = "Generate teleburn addresses")]
  Teleburn(teleburn::Teleburn),
  #[command(about = "Display satoshi traits")]
  Traits(traits::Traits),
  #[command(subcommand, about = "Wallet commands")]
  Wallet(wallet::Wallet),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self {
      Self::Decode(decode) => decode.run(),
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(options),
      Self::Index(index) => index.run(options),
      Self::List(list) => list.run(options),
      Self::Parse(parse) => parse.run(),
      Self::Preview(preview) => preview.run(),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(options, index, handle)
      }
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Supply => supply::run(),
      Self::Teleburn(teleburn) => teleburn.run(),
      Self::Traits(traits) => traits.run(),
      Self::Wallet(wallet) => wallet.run(options),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Empty {}

pub(crate) trait Output: Send {
  fn print_json(&self);
}

impl<T> Output for T
where
  T: Serialize + Send,
{
  fn print_json(&self) {
    serde_json::to_writer_pretty(io::stdout(), self).ok();
    println!();
  }
}

pub(crate) type SubcommandResult = Result<Box<dyn Output>>;
