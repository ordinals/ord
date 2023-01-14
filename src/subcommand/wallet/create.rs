use super::*;

#[derive(Serialize)]
struct Output {
  seed_phrase: Mnemonic,
}

pub(crate) fn run(options: Options) -> Result {
  let mut entropy = [0; 16];
  rand::thread_rng().fill_bytes(&mut entropy);

  let seed_phrase = Mnemonic::from_entropy(&entropy)?;

  assert_eq!(seed_phrase.word_count(), 12);

  initialize_wallet(&options, &entropy)?;

  serde_json::to_writer_pretty(io::stdout(), &Output { seed_phrase })?;

  Ok(())
}
