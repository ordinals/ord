use {super::*, std::io::Read};

#[derive(Debug, Parser)]
#[clap(group(ArgGroup::new("restore_source").required(true).args(&["from_descriptor", "from_mnemonic"])))]
pub(crate) struct Restore {
  #[arg(long, conflicts_with_all = &["from_mnemonic", "passphrase"], help = "Restore wallet from a Bitcoin Core <DESCRIPTOR> passed through STDIN.")]
  from_descriptor: bool,
  #[arg(long, help = "Restore wallet from <MNEMONIC>.")]
  from_mnemonic: Option<Mnemonic>,
  #[arg(
    long,
    default_value = "",
    help = "Use <PASSPHRASE> when deriving wallet"
  )]
  pub(crate) passphrase: String,
}

impl Restore {
  pub(crate) fn run(self, mut wallet: Wallet) -> SubcommandResult {
    match (self.from_descriptor, self.from_mnemonic) {
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
        wallet.initialize(mnemonic.to_seed(self.passphrase))?;
      }
      _ => {
        bail!("either a descriptor or a mnemonic is required.");
      }
    }

    Ok(None)
  }
}
