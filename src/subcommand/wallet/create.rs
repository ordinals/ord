use super::*;

#[derive(Serialize)]
struct Output {
  mnemonic: Mnemonic,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let mut entropy = [0; 16];
  rand::thread_rng().fill_bytes(&mut entropy);

  let mnemonic = Mnemonic::from_entropy(&entropy)?;

  initialize_wallet(&options, mnemonic.to_seed(""))?;

  Ok(Box::new(Output { mnemonic }))
}
