use super::*;

mod identify;
mod list;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  List,
  Identify,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::List => list::run(options),
      Self::Identify => identify::run(options),
    }
  }
}

#[cfg(test)]
mod tests {}
