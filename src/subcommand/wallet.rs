use {
  super::*,
  crate::wallet::{Wallet, batch, wallet_constructor::WalletConstructor},
  bdk::KeychainKind,
  bitcoin::Psbt,
  shared_args::SharedArgs,
};

pub mod addresses;
pub mod balance;
mod batch_command;
pub mod burn;
pub mod cardinals;
pub mod create;
pub mod descriptors;
pub mod inscribe;
pub mod inscriptions;
mod label;
pub mod mint;
pub mod offer;
pub mod outputs;
pub mod pending;
pub mod receive;
pub mod restore;
pub mod resume;
pub mod runics;
pub mod sats;
pub mod send;
mod shared_args;
pub mod sign;
pub mod split;
pub mod sweep;
pub mod transactions;

#[derive(Debug, Parser)]
pub(crate) struct WalletCommand {
  #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
  pub(crate) name: String,
  #[arg(
    long,
    help = "Use ord running at <SERVER_URL>. [default: http://localhost:80]"
  )]
  pub(crate) server_url: Option<Url>,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

#[derive(Debug, Parser)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Subcommand {
  #[command(about = "List addresses")]
  Addresses,
  #[command(about = "Get balance")]
  Balance,
  #[command(about = "Create inscriptions and runes")]
  Batch(batch_command::Batch),
  #[command(about = "Burn an inscription")]
  Burn(burn::Burn),
  #[command(about = "List unspent cardinal outputs")]
  Cardinals,
  #[command(about = "Create new wallet")]
  Create(create::Create),
  #[command(about = "List descriptors")]
  Descriptors,
  #[command(about = "Create inscription")]
  Inscribe(inscribe::Inscribe),
  #[command(about = "List inscriptions")]
  Inscriptions,
  #[command(about = "Export output labels")]
  Label,
  #[command(about = "Mint a rune")]
  Mint(mint::Mint),
  #[command(subcommand, about = "Offer commands")]
  Offer(offer::Offer),
  #[command(about = "List unspent outputs")]
  Outputs(outputs::Outputs),
  #[command(about = "List pending etchings")]
  Pending(pending::Pending),
  #[command(about = "Generate receive address")]
  Receive(receive::Receive),
  #[command(about = "Restore wallet")]
  Restore(restore::Restore),
  #[command(about = "Resume pending etchings")]
  Resume(resume::Resume),
  #[command(about = "List unspent runic outputs")]
  Runics,
  #[command(about = "List satoshis")]
  Sats(sats::Sats),
  #[command(about = "Send sat or inscription")]
  Send(send::Send),
  #[command(about = "Sign message")]
  Sign(sign::Sign),
  #[command(about = "Split outputs")]
  Split(split::Split),
  #[command(about = "Sweep assets from private key")]
  Sweep(sweep::Sweep),
  #[command(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
}

impl WalletCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self.subcommand {
      Subcommand::Create(create) => return create.run(&settings, &self.name),
      Subcommand::Restore(restore) => return restore.run(&settings, &self.name),
      _ => {}
    };

    let wallet = WalletConstructor::construct(
      self.name.clone(),
      settings.clone(),
      self
        .server_url
        .as_ref()
        .map(Url::as_str)
        .or(settings.server_url())
        .unwrap_or("http://127.0.0.1:80")
        .parse::<Url>()
        .context("invalid server URL")?,
    )?;

    match self.subcommand {
      Subcommand::Addresses => addresses::run(wallet),
      Subcommand::Balance => balance::run(wallet),
      Subcommand::Batch(batch) => batch.run(wallet),
      Subcommand::Burn(burn) => burn.run(wallet),
      Subcommand::Cardinals => cardinals::run(wallet),
      Subcommand::Create(_) | Subcommand::Restore(_) => unreachable!(),
      Subcommand::Descriptors => descriptors::run(wallet),
      Subcommand::Inscribe(inscribe) => inscribe.run(wallet),
      Subcommand::Inscriptions => inscriptions::run(wallet),
      Subcommand::Label => label::run(wallet),
      Subcommand::Mint(mint) => mint.run(wallet),
      Subcommand::Offer(offer) => offer.run(wallet),
      Subcommand::Outputs(outputs) => outputs.run(wallet),
      Subcommand::Pending(pending) => pending.run(wallet),
      Subcommand::Receive(receive) => receive.run(wallet),
      Subcommand::Resume(resume) => resume.run(wallet),
      Subcommand::Runics => runics::run(wallet),
      Subcommand::Sats(sats) => sats.run(wallet),
      Subcommand::Send(send) => send.run(wallet),
      Subcommand::Sign(sign) => sign.run(wallet),
      Subcommand::Split(split) => split.run(wallet),
      Subcommand::Sweep(sweep) => sweep.run(wallet),
      Subcommand::Transactions(transactions) => transactions.run(wallet),
    }
  }

  fn parse_metadata(cbor: Option<PathBuf>, json: Option<PathBuf>) -> Result<Option<Vec<u8>>> {
    match (cbor, json) {
      (None, None) => Ok(None),
      (Some(path), None) => {
        let cbor = fs::read(path)?;
        let _value: Value = ciborium::from_reader(Cursor::new(cbor.clone()))
          .context("failed to parse CBOR metadata")?;

        Ok(Some(cbor))
      }
      (None, Some(path)) => {
        let value: serde_json::Value =
          serde_json::from_reader(File::open(path)?).context("failed to parse JSON metadata")?;
        let mut cbor = Vec::new();
        ciborium::into_writer(&value, &mut cbor)?;

        Ok(Some(cbor))
      }
      (Some(_), Some(_)) => panic!(),
    }
  }
}
