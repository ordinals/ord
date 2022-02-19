use super::*;

mod epochs;
mod find;
mod index;
mod info;
mod list;
mod name;
mod range;
mod supply;
mod traits;

#[derive(StructOpt)]
pub(crate) enum Subcommand {
  Epochs,
  Find(find::Find),
  Index,
  List(list::List),
  Name(name::Name),
  Range(range::Range),
  Supply,
  Info,
  Traits(traits::Traits),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(options),
      Self::Index => index::run(options),
      Self::List(list) => list.run(options),
      Self::Name(name) => name.run(),
      Self::Range(range) => range.run(),
      Self::Supply => supply::run(),
      Self::Info => info::run(),
      Self::Traits(traits) => traits.run(),
    }
  }
}
