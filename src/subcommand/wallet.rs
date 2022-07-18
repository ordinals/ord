use super::*;

mod fund;
mod init;

fn get_key() -> Result<impl DerivableKey<Segwitv0> + Clone> {
  Ok((
    Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")?,
    None,
  ))
}

fn get_wallet() -> Result<bdk::wallet::Wallet<SqliteDatabase>> {
  let db_path = PathBuf::from(format!("{}/ord", data_dir().unwrap().display()));

  if !db_path.exists() {
    fs::create_dir_all(&db_path)?;
  }

  Ok(bdk::wallet::Wallet::new(
    Bip84(get_key()?, KeychainKind::External),
    None,
    Network::Signet,
    SqliteDatabase::new(format!("{}/wallet.sqlite", db_path.display())),
  )?)
}

#[derive(Parser)]
pub(crate) enum Wallet {
  Init,
  Fund,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Init => init::run(options),
      Self::Fund => fund::run(),
    }
  }
}
