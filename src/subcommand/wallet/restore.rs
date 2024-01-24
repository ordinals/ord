use {super::*, bitcoincore_rpc::bitcoincore_rpc_json::Descriptor};

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[arg(long, help = "Restore wallet from <DESCRIPTOR>.")]
  descriptor: String,
  #[arg(long, help = "Restore wallet from <MNEMONIC>.")]
  mnemonic: Mnemonic,
  #[arg(
    long,
    default_value = "",
    help = "Use <PASSPHRASE> when deriving wallet"
  )]
  pub(crate) passphrase: String,
}

impl Restore {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    wallet.initialize(self.mnemonic.to_seed(self.passphrase))?;

    Ok(None)
  }
}
