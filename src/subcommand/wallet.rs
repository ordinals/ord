use super::*;

mod identify;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Identify,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Identify => identify::run(options),
    }
  }
}

#[cfg(test)]
mod tests {}
