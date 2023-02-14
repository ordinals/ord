use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[clap(help = "Restore wallet from <MNEMONIC>")]
  mnemonic: Mnemonic,
  #[clap(long, help = "Use <PASSPHRASE> when deriving wallet")]
  pub(crate) passphrase: Option<String>,
}

impl Restore {
  pub(crate) fn run(self, options: Options) -> Result {
    initialize_wallet(
      &options,
      self.mnemonic.to_seed(self.passphrase.unwrap_or("".into())),
    )?;

    Ok(())
  }
}
