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
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    initialize_wallet(&options, self.mnemonic.to_seed(self.passphrase))?;
    Ok(Box::new(Empty {}))
  }
}
