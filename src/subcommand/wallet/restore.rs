use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[clap(value_enum, long, help = "Restore wallet from <SOURCE> on stdin.")]
  from: Source,
  #[arg(long, help = "Use <PASSPHRASE> when deriving wallet")]
  pub(crate) passphrase: Option<String>,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum Source {
  Descriptor,
  Mnemonic,
}

impl Restore {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(!wallet.exists()?, "wallet `{}` already exists", wallet.name);

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    match self.from {
      Source::Descriptor => {
        ensure!(
          self.passphrase.is_none(),
          "descriptor does not take a passphrase"
        );
        let wallet_descriptors: ListDescriptorsResult = serde_json::from_str(&buffer)?;
        wallet.initialize_from_descriptors(wallet_descriptors.descriptors)?;
      }
      Source::Mnemonic => {
        let mnemonic = Mnemonic::parse_normalized(&buffer)?;
        wallet.initialize(mnemonic.to_seed(self.passphrase.unwrap_or_default()))?;
      }
    }

    Ok(None)
  }
}
