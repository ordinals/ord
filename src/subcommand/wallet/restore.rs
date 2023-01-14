use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Restore {
  #[clap(help = "Restore wallet from <SEED_PHRASE>")]
  seed_phrase: Mnemonic,
}

impl Restore {
  pub(crate) fn run(self, options: Options) -> Result {
    let entropy = self.seed_phrase.to_entropy();

    initialize_wallet(&options, &entropy)?;

    Ok(())
  }
}
