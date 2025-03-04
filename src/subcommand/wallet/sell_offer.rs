use super::*;

pub mod create;

#[derive(Debug, Parser)]
pub(crate) enum SellOffer {
  #[command(about = "Create offer to sell rune")]
  Create(create::Create),
}

impl SellOffer {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    match self {
      Self::Create(create) => create.run(wallet),
    }
  }
}
