use super::*;

mod epochs;
mod find;
mod list;
mod name;
mod range;
mod supply;
mod traits;

#[derive(StructOpt)]
pub(crate) enum Subcommand {
  Epochs,
  Find(find::Find),
  Name(name::Name),
  List(list::List),
  Range(range::Range),
  Supply,
  Traits(traits::Traits),
}

impl Subcommand {
  pub(crate) fn run(self, index_size: Option<usize>) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(index_size),
      Self::Name(name) => name.run(),
      Self::List(list) => list.run(index_size),
      Self::Range(range) => range.run(),
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
    }
  }
}
