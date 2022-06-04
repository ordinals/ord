use super::*;

mod epochs;
mod find;
mod generate_paper_wallet;
mod index;
mod info;
mod list;
mod mint;
mod name;
mod range;
mod server;
mod supply;
mod traits;
mod verify;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Epochs,
  Find(find::Find),
  Index,
  Info,
  List(list::List),
  Mint(mint::Mint),
  Name(name::Name),
  Range(range::Range),
  Server(server::Server),
  Supply,
  Traits(traits::Traits),
  GeneratePaperWallet,
  Verify(verify::Verify),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::GeneratePaperWallet => generate_paper_wallet::run(),
      Self::Find(find) => find.run(options),
      Self::Index => index::run(options),
      Self::List(list) => list.run(options),
      Self::Name(name) => name.run(),
      Self::Mint(mint) => mint.run(),
      Self::Range(range) => range.run(),
      Self::Supply => supply::run(),
      Self::Server(server) => server.run(options),
      Self::Info => info::run(options),
      Self::Traits(traits) => traits.run(),
      Self::Verify(verify) => verify.run(),
    }
  }
}
