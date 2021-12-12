use super::*;

#[derive(StructOpt)]
pub enum Arguments {
  Catalog,
  Supply,
}

impl Arguments {
  pub fn run(self) -> Result<()> {
    match self {
      Self::Catalog => catalog::run(),
      Self::Supply => supply::run(),
    }
  }
}
