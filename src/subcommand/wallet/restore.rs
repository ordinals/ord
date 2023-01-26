use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[clap(help = "Restore wallet from <MNEMONIC>")]
  mnemonic: Mnemonic,
}

impl Restore {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    initialize_wallet(&options, self.mnemonic.to_seed(""))?;

    Ok(Box::new(Empty {}))
  }
}
