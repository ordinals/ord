use {
  super::*,
  crate::wallet::{
    inscribe::{Batch, Batchfile, Mode},
    Wallet,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::ListDescriptorsResult,
  reqwest::Url,
};

pub mod balance;
pub mod cardinals;
pub mod create;
pub mod dump;
pub mod etch;
pub mod inscribe;
pub mod inscriptions;
pub mod outputs;
pub mod receive;
pub mod restore;
pub mod sats;
pub mod send;
pub mod transactions;

#[derive(Debug, Parser)]
pub(crate) struct WalletCommand {
  #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
  pub(crate) name: String,
  #[arg(long, alias = "nosync", help = "Do not update index.")]
  pub(crate) no_sync: bool,
  #[arg(
    long,
    default_value = "http://127.0.0.1:80",
    help = "Use ord running at <SERVER_URL>."
  )]
  pub(crate) server_url: Url,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

#[derive(Debug, Parser)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Subcommand {
  #[command(about = "Get wallet balance")]
  Balance,
  #[command(about = "Create new wallet")]
  Create(create::Create),
  #[command(about = "Dump wallet descriptors")]
  Dump,
  #[command(about = "Create rune")]
  Etch(etch::Etch),
  #[command(about = "Create inscription")]
  Inscribe(inscribe::Inscribe),
  #[command(about = "List wallet inscriptions")]
  Inscriptions,
  #[command(about = "Generate receive address")]
  Receive,
  #[command(about = "Restore wallet")]
  Restore(restore::Restore),
  #[command(about = "List wallet satoshis")]
  Sats(sats::Sats),
  #[command(about = "Send sat or inscription")]
  Send(send::Send),
  #[command(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
  #[command(about = "List all unspent outputs in wallet")]
  Outputs,
  #[command(about = "List unspent cardinal outputs in wallet")]
  Cardinals,
}

impl WalletCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self.subcommand {
      Subcommand::Create(create) => return create.run(self.name, &settings),
      Subcommand::Restore(restore) => return restore.run(self.name, &settings),
      _ => {}
    };

    let wallet = Wallet::build(
      self.name.clone(),
      self.no_sync,
      settings.clone(),
      self.server_url,
    )?;

    match self.subcommand {
      Subcommand::Balance => balance::run(wallet),
      Subcommand::Dump => dump::run(wallet),
      Subcommand::Etch(etch) => etch.run(wallet),
      Subcommand::Inscribe(inscribe) => inscribe.run(wallet),
      Subcommand::Inscriptions => inscriptions::run(wallet),
      Subcommand::Receive => receive::run(wallet),
      Subcommand::Sats(sats) => sats.run(wallet),
      Subcommand::Send(send) => send.run(wallet),
      Subcommand::Transactions(transactions) => transactions.run(wallet),
      Subcommand::Outputs => outputs::run(wallet),
      Subcommand::Cardinals => cardinals::run(wallet),
      Subcommand::Create(_) | Subcommand::Restore(_) => unreachable!(),
    }
  }
}
