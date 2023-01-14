use super::*;

#[derive(Serialize)]
struct Output {
  mnemonic: Mnemonic,
}

pub(crate) fn run(options: Options) -> Result {
  let mut entropy = [0; 16];
  rand::thread_rng().fill_bytes(&mut entropy);

  let mnemonic = Mnemonic::from_entropy(&entropy)?;

  // TODO:
  // - actually use PBKDF2 as prescribed in BIP-39 to generate seed

  initialize_wallet(&options, &entropy)?;

  serde_json::to_writer_pretty(io::stdout(), &Output { mnemonic })?;

  Ok(())
}
