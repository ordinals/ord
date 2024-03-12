use {
  super::*,
  bitcoin::secp256k1::rand::{self, RngCore},
};

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub mnemonic: Mnemonic,
  pub passphrase: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(
    long,
    default_value = "",
    help = "Use <PASSPHRASE> to derive wallet seed."
  )]
  pub(crate) passphrase: String,
}

impl Create {
  pub(crate) fn run(self, name: String, settings: &Settings) -> SubcommandResult {
    let mut entropy = [0; 16];
    rand::thread_rng().fill_bytes(&mut entropy);

    let mnemonic = Mnemonic::from_entropy(&entropy)?;

    Wallet::initialize(name, settings, mnemonic.to_seed(&self.passphrase))?;

    Ok(Some(Box::new(Output {
      mnemonic,
      passphrase: Some(self.passphrase),
    })))
  }
}
