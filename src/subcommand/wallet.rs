use super::*;

mod fund;
mod init;
mod utxos;

use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};
use bdk::blockchain::ConfigurableBlockchain;
use bdk::wallet::{wallet_name_from_descriptor, SyncOptions};

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
    Network::Regtest,
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
      url: options
        .rpc_url
        .ok_or_else(|| anyhow!("This command requires `--rpc-url`"))?,
      auth: options
        .cookie_file
        .map(|path| Auth::Cookie { file: path })
        .unwrap_or(Auth::None),
      network: Network::Regtest,
      wallet_name: wallet_name_from_descriptor(
        Bip84(key, KeychainKind::External),
        None,
        Network::Regtest,
        &Secp256k1::new(),
      )?,
      skip_blocks: None,
    })?,
    SyncOptions::default(),
  )?;

  Ok(wallet)
}

#[derive(Parser)]
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
