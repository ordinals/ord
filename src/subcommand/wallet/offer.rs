use super::*;

mod create;

#[derive(Debug, Parser)]
pub(crate) enum Offer {
  #[command(about = "Create an offer to buy an inscription")]
   Create(create::Create),
}

impl Offer {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    match self {
      Self::Create(create) => create.run(wallet),
    }
  }
}
