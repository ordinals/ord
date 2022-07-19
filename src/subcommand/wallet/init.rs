use super::*;

pub(crate) fn run() -> Result {
  let path = data_dir()
    .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
    .join("ord");

  if path.exists() {
    return Err(anyhow!("Wallet already exists."));
  }

  fs::create_dir_all(&path)?;

  let seed = Mnemonic::generate_in_with(&mut rand::thread_rng(), Language::English, 12)?;

  fs::write(path.join("entropy"), seed.to_entropy())?;

  bdk::wallet::Wallet::new(
    Bip84((seed, None), KeychainKind::External),
    None,
    Network::Signet,
    SqliteDatabase::new(
      path
        .join("wallet.sqlite")
        .to_str()
        .ok_or_else(|| anyhow!("Failed to convert path to str"))?
        .to_string(),
    ),
  )?;

  eprintln!("Wallet initialized.");

  Ok(())
}
