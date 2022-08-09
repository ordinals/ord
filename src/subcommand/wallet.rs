use super::*;

mod fund;
mod init;
mod utxos;

fn get_wallet(options: Options) -> Result<bdk::wallet::Wallet<SqliteDatabase>> {
  let path = data_dir()
    .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
    .join("ord");

  if !path.exists() {
    return Err(anyhow!("Wallet doesn't exist."));
  }

  let key = (
    Mnemonic::from_entropy(&fs::read(path.join("entropy"))?)?,
    None,
  );

  let wallet = bdk::wallet::Wallet::new(
    Bip84(key.clone(), KeychainKind::External),
    None,
    options.network,
    SqliteDatabase::new(
      path
        .join("wallet.sqlite")
        .to_str()
        .ok_or_else(|| anyhow!("Failed to convert path to str"))?
        .to_string(),
    ),
  )?;

  wallet.sync(
    &RpcBlockchain::from_config(&RpcConfig {
      url: options.rpc_url(),
      auth: Auth::Cookie {
        file: options.cookie_file()?,
      },
      network: options.network,
      wallet_name: wallet_name_from_descriptor(
        Bip84(key, KeychainKind::External),
        None,
        options.network,
        &Secp256k1::new(),
      )?,
      skip_blocks: None,
    })?,
    SyncOptions::default(),
  )?;

  Ok(wallet)
}

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Init,
  Fund,
  Utxos,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Init => init::run(options),
      Self::Fund => fund::run(options),
      Self::Utxos => utxos::run(options),
    }
  }
}
