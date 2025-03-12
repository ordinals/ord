use super::*;

pub mod accept;
pub mod create;

#[derive(Debug, Parser)]
pub(crate) enum Offer {
  #[command(about = "Accept offer to buy inscription (DEPRECATED)")]
  Accept(accept::Accept),
  #[command(about = "Create offer to buy inscription (DEPRECATED)")]
  Create(create::Create),
}

impl Offer {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    match self {
      Self::Accept(accept) => accept.run(wallet),
      Self::Create(create) => create.run(wallet),
    }
  }
}
