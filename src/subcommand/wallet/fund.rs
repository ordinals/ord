use super::*;

pub(crate) fn run() -> Result {
  let path = data_dir()
    .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
    .join("ord");

  if !path.exists() {
    return Err(anyhow!("Wallet doesn't exist."));
  }

  let entropy = fs::read(path.join("entropy"))?;

  let wallet = bdk::wallet::Wallet::new(
    Bip84(
      (Mnemonic::from_entropy(&entropy)?, None),
      KeychainKind::External,
    ),
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

  println!("{}", wallet.get_address(LastUnused)?);

  Ok(())
}
