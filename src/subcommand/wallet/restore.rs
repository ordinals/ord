use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[clap(help = "Restore wallet from <MNEMONIC>")]
  mnemonic: Mnemonic,
  #[clap(
    long,
    default_value = "",
    help = "Use <PASSPHRASE> when deriving wallet"
  )]
  pub(crate) passphrase: String,
}

impl Restore {
  pub(crate) fn run(self, _options: Options) -> Result {
    bail!(
      "Descriptor wallets are not supported in Litecoincore 21.2.1, copy your wallet.dat into \
      your Litecoincore data directory."
    );

    // initialize_wallet(&options, self.mnemonic.to_seed(self.passphrase))?;
    //
    // Ok(())
  }
}
