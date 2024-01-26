use super::*;

#[derive(Debug, Parser)]
#[clap(group(
  ArgGroup::new("source").required(true).args(&["descriptor", "mnemonic"]))
)]
pub(crate) struct Restore {
  #[arg(long, help = "Restore wallet from <DESCRIPTOR> from stdin.")]
  descriptor: bool,
  #[arg(long, help = "Restore wallet from <MNEMONIC>.")]
  mnemonic: Option<Mnemonic>,
  #[arg(
    long,
    requires = "mnemonic",
    help = "Use <PASSPHRASE> when deriving wallet"
  )]
  pub(crate) passphrase: Option<String>,
}

impl Restore {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      !wallet.exists()?,
      "cannot restore because wallet named `{}` already exists",
      wallet.name
    );

    match (self.descriptor, self.mnemonic) {
      (true, None) => {
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer)?;

        let wallet_descriptors: BitcoinCoreDescriptors = serde_json::from_slice(&buffer)?;

        wallet.initialize_from_descriptors(wallet_descriptors.descriptors)?;
      }
      (false, Some(mnemonic)) => {
        wallet.initialize(mnemonic.to_seed(self.passphrase.unwrap_or_default()))?;
      }
      _ => {
        bail!("either a descriptor or a mnemonic is required.");
      }
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
