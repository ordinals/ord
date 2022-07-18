use super::*;

pub(crate) fn run() -> Result {
  let key = get_key()?;

  let wallet = Wallet::new(
    Bip84(key.clone(), KeychainKind::External),
    Some(Bip84(key.clone(), KeychainKind::Internal)),
    Network::Regtest,
    SqliteDatabase::new("foo".into()),
  )?;

  println!("{}", wallet.get_address(LastUnused)?);

  Ok(())
}
