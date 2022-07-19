use super::*;

mod fund;
mod init;

fn get_wallet() -> Result<bdk::wallet::Wallet<SqliteDatabase>> {
  let path = data_dir()
    .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
    .join("ord");

  if !path.exists() {
    return Err(anyhow!("Wallet doesn't exist."));
  }

  let entropy = fs::read(path.join("seed.txt"))?;

  Ok(bdk::wallet::Wallet::new(
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
