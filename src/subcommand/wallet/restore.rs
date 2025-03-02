use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[arg(long, help = "Use <PASSPHRASE> when deriving wallet.")]
  pub(crate) passphrase: Option<String>,
}

impl Restore {
  pub(crate) fn run(self, settings: &Settings, name: &str) -> SubcommandResult {
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer)?;

    let mnemonic = Mnemonic::from_str(&buffer)?;

    Wallet::create(
      settings,
      name,
      mnemonic.to_seed(self.passphrase.unwrap_or_default()),
    )?;

    Ok(None)
  }
}
