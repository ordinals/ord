use super::*;

#[derive(Debug, Parser)]
#[clap(group(
  ArgGroup::new("source").required(true).args(&["descriptor", "mnemonic"]))
)]
pub(crate) struct Restore {
  #[arg(long, help = "Restore wallet from <DESCRIPTOR> from stdin.")]
  descriptor: bool,
  #[arg(long, help = "Restore wallet from <MNEMONIC> from stdin.")]
  mnemonic: bool,
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

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    if self.descriptor {
      let wallet_descriptors: ListDescriptorsResult = serde_json::from_str(&buffer)?;
      wallet.initialize_from_descriptors(wallet_descriptors.descriptors)?;
    } else if self.mnemonic {
      let mnemonic = Mnemonic::parse_normalized(&buffer)?;
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
      Arguments::try_parse_from(["ord", "wallet", "restore", "--descriptor", "--mnemonic",])
        .unwrap_err()
        .to_string(),
      ".*--descriptor.*cannot be used with.*--mnemonic.*"
    );
  }
}
