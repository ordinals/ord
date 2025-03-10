use super::*;

pub mod accept;
pub mod create;

#[derive(Debug, Parser)]
pub(crate) enum SellOffer {
  #[command(about = "Accept offer to sell rune")]
  Accept(accept::Accept),
  #[command(about = "Create offer to sell rune")]
  Create(create::Create),
}

impl SellOffer {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    match self {
      Self::Accept(accept) => accept.run(wallet),
      Self::Create(create) => create.run(wallet),
    }
  }
}
