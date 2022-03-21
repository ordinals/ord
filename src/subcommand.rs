use super::*;

mod epochs;
mod index;
mod info;
mod list;
mod name;
mod range;
mod server;
mod supply;
mod traits;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Epochs,
  Index,
  List(list::List),
  Name(name::Name),
  Range(range::Range),
  Supply,
  Server(server::Server),
  Info,
  Traits(traits::Traits),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::Index => index::run(options),
      Self::List(list) => list.run(options),
      Self::Name(name) => name.run(),
      Self::Range(range) => range.run(),
      Self::Supply => supply::run(),
      Self::Server(server) => server.run(options),
      Self::Info => info::run(options),
      Self::Traits(traits) => traits.run(),
    }
  }
}
