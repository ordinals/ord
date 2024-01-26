use {super::*, std::io::Read};

#[derive(Debug, Parser)]
#[clap(group(
  ArgGroup::new("source").required(true).args(&["descriptor", "mnemonic"]))
)]
pub(crate) struct Restore {
  #[arg(
    long,
    conflicts_with_all = &["mnemonic", "passphrase"],
    help = "Restore wallet from <DESCRIPTOR> from stdin."
  )]
  descriptor: bool,
  #[arg(long, help = "Restore wallet from <MNEMONIC>.")]
  mnemonic: Option<Mnemonic>,
  #[arg(long, help = "Use <PASSPHRASE> when deriving wallet")]
  pub(crate) passphrase: Option<String>,
}

impl Restore {
  pub(crate) fn run(self, mut wallet: Wallet) -> SubcommandResult {
    match (self.descriptor, self.mnemonic) {
      (true, None) => {
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer)?;

        let wallet_descriptors: BitcoinCoreDescriptors = serde_json::from_slice(&buffer)?;

        wallet.name = wallet_descriptors.wallet_name;

        if wallet.exists()? {
          bail!(
            "cannot restore because wallet named `{}` already exists",
            wallet.name
          );
        }

        wallet.initialize_from_descriptors(wallet_descriptors.descriptors)?;
      }
      (false, Some(mnemonic)) => {
        if wallet.exists()? {
          bail!(
            "cannot restore because wallet named `{}` already exists",
            wallet.name
          );
        }

        wallet.initialize(mnemonic.to_seed(self.passphrase.unwrap_or_default()))?;
      }
      _ => {
        bail!("either a descriptor or a mnemonic is required.");
      }
    }

    Ok(None)
  }
}
