use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Timestamp(bitcoincore_rpc::json::Timestamp);

impl FromStr for Timestamp {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if s == "now" {
      Self(bitcoincore_rpc::json::Timestamp::Now)
    } else {
      Self(bitcoincore_rpc::json::Timestamp::Time(s.parse()?))
    })
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[arg(long, help = "Use <PASSPHRASE> when deriving wallet.")]
  pub(crate) passphrase: Option<String>,
  #[arg(
    long,
    help = "Scan chain from <TIMESTAMP> onwards. Can be a unix timestamp in \
    seconds or the string `now`, to skip scanning"
  )]
  pub(crate) timestamp: Option<Timestamp>,
}

impl Restore {
  pub(crate) fn run(self, name: String, settings: &Settings) -> SubcommandResult {
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer)?;

    let mnemonic = Mnemonic::from_str(&buffer)?;

    Wallet::create(
      name,
      settings,
      mnemonic.to_seed(self.passphrase.unwrap_or_default()),
      self
        .timestamp
        .unwrap_or(Timestamp(bitcoincore_rpc::json::Timestamp::Time(0)))
        .0,
    )?;

    Ok(None)
  }
}
