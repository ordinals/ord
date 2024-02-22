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
  pub(crate) fn run(self, name: String, settings: &Settings) -> SubcommandResult {
    ensure!(
      !settings
        .bitcoin_rpc_client(None)?
        .list_wallet_dir()?
        .iter()
        .any(|wallet_name| wallet_name == &name),
      "wallet `{}` already exists",
      name
    );

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    match self.from {
      Source::Descriptor => {
        ensure!(
          self.passphrase.is_none(),
          "descriptor does not take a passphrase"
        );
        let wallet_descriptors: ListDescriptorsResult = serde_json::from_str(&buffer)?;
        Wallet::initialize_from_descriptors(name, settings, wallet_descriptors.descriptors)?;
      }
      Source::Mnemonic => {
        let mnemonic = Mnemonic::from_str(&buffer)?;
        Wallet::initialize(
          name,
          settings,
          mnemonic.to_seed(self.passphrase.unwrap_or_default()),
        )?;
      }
    }

    Ok(None)
  }
}
