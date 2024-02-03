use {
  super::*,
  std::{borrow::Cow, io, io::Write},
};

#[derive(Debug, Parser)]
#[clap(group(
  ArgGroup::new("source").required(true).args(&["descriptor", "mnemonic"]))
)]
pub(crate) struct Restore {
  #[arg(long, help = "Restore wallet from <DESCRIPTOR> from stdin.")]
  descriptor: bool,
  #[arg(long, help = "Restore wallet from <MNEMONIC>.")]
  mnemonic: Option<Option<Mnemonic>>,
  #[arg(
    long,
    requires = "mnemonic",
    help = "Use <PASSPHRASE> when deriving wallet"
  )]
  pub(crate) passphrase: Option<String>,
}

impl Restore {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(!wallet.exists()?, "wallet `{}` already exists", wallet.name);

    if self.descriptor {
      let mut buffer = Vec::new();
      io::stdin().read_to_end(&mut buffer)?;

      let wallet_descriptors: ListDescriptorsResult = serde_json::from_slice(&buffer)?;

      wallet.initialize_from_descriptors(wallet_descriptors.descriptors)?;
    } else if let Some(Some(mnemonic)) = self.mnemonic {
      wallet.initialize(mnemonic.to_seed(self.passphrase.unwrap_or_default()))?;
    } else if let Some(None) = self.mnemonic {
      let mut buffer = String::new();
      io::stdout().write_all(b"Please input your seed phrase:")?;
      io::stdin().read_to_string(&mut buffer)?;
      let buffer = Cow::<str>::Owned(buffer);
      let mnemonic: Mnemonic = Mnemonic::parse(buffer)?;
      wallet.initialize(mnemonic.to_seed(self.passphrase.unwrap_or_default()))?
    } else {
      unreachable!()
    }

    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn descriptor_and_mnemonic_conflict() {
    assert_regex_match!(
      Arguments::try_parse_from([
        "ord",
        "wallet",
        "restore",
        "--descriptor",
        "--mnemonic",
        "oil oil oil oil oil oil oil oil oil oil oil oil"
      ])
      .unwrap_err()
      .to_string(),
      ".*--descriptor.*cannot be used with.*--mnemonic.*"
    );
  }
}
