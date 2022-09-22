use super::*;

mod epochs;
mod find;
mod index;
mod info;
mod list;
mod parse;
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
  Parse(parse::Parse),
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
      Self::Parse(parse) => parse.run(),
      Self::Range(range) => range.run(),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&options)?);
        let handle = Handle::new();

        {
          let handle = handle.clone();
          ctrlc::set_handler(move || {
            handle.graceful_shutdown(Some(Duration::from_millis(100)));

            let interrupts = INTERRUPTS.fetch_add(1, atomic::Ordering::Relaxed);

            if interrupts > 5 {
              process::exit(1);
            }
          })
          .expect("Error setting ctrl-c handler");
        }

        server.run(options, index, handle)
      }
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
      Self::Wallet(wallet) => wallet.run(options),
    }
  }
}
