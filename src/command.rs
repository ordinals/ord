use super::*;

mod epochs;
mod find;
mod list;
mod name;
mod range;
mod supply;
mod traits;

#[derive(StructOpt)]
pub(crate) enum Command {
  Epochs,
  Find(find::Find),
  Name(name::Name),
  List(list::List),
  Range(range::Range),
  Supply,
  Traits(traits::Traits),
}

impl Command {
  pub(crate) fn run(self) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(),
      Self::Name(name) => name.run(),
      Self::List(list) => list.run(),
      Self::Range(range) => range.run(),
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
    }
  }
}
