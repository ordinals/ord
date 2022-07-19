use super::*;

mod epochs;
mod find;
mod generate_paper_wallets;
mod generate_private_key;
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
mod wallet;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Epochs,
  Find(find::Find),
  GeneratePaperWallets,
  GeneratePrivateKey,
  Index,
  Info,
  List(list::List),
  Mint(mint::Mint),
  Name(name::Name),
  Range(range::Range),
  Server(server::Server),
  Supply,
  Traits(traits::Traits),
  Verify(verify::Verify),
  #[clap(subcommand)]
  Wallet(wallet::Wallet),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(options),
      Self::GeneratePaperWallets => generate_paper_wallets::run(),
      Self::GeneratePrivateKey => generate_private_key::run(),
      Self::Index => index::run(options),
      Self::Info => info::run(options),
      Self::List(list) => list.run(options),
      Self::Mint(mint) => mint.run(),
      Self::Name(name) => name.run(),
      Self::Range(range) => range.run(),
      Self::Server(server) => server.run(options),
      Self::Supply => supply::run(),
      Self::Traits(traits) => traits.run(),
      Self::Verify(verify) => verify.run(),
      Self::Wallet(wallet) => wallet.run(),
    }
  }
}
