use super::*;

#[derive(Serialize)]
struct Output {
  mnemonic: Mnemonic,
  passphrase: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[clap(long, help = "Use <PASSPHRASE> when deriving wallet")]
  pub(crate) passphrase: Option<String>,
}

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    let mut entropy = [0; 16];
    rand::thread_rng().fill_bytes(&mut entropy);

    let mnemonic = Mnemonic::from_entropy(&entropy)?;

    initialize_wallet(
      &options,
      mnemonic.to_seed(self.passphrase.clone().unwrap_or("".into())),
    )?;

    print_json(Output {
      mnemonic,
      passphrase: self.passphrase,
    })?;

    Ok(())
  }
}
