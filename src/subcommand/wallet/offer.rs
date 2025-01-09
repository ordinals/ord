use super::*;

mod accept;
mod create;
mod view;

#[derive(Debug, Parser)]
pub(crate) enum Offer {
  #[command(about = "Accept an offer to buy an inscription")]
  Accept(accept::Accept),
  #[command(about = "Create an offer to buy an inscription")]
  Create(create::Create),
  #[command(about = "View an offer to buy an inscription")]
  View(view::View),
}

impl Offer {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    match self {
      Self::Accept(accept) => accept.run(wallet),
      Self::Create(create) => create.run(wallet),
      Self::View(view) => view.run(wallet),
    }
  }
}
