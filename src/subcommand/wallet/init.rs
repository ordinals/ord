use super::*;

use bdk::{
  blockchain::{
    rpc::{Auth, RpcBlockchain, RpcConfig},
    ConfigurableBlockchain,
  },
  database::SqliteDatabase,
  keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, GeneratableKey,
  },
  miniscript::miniscript::Segwitv0,
  template::Bip84,
  wallet::{wallet_name_from_descriptor, SyncOptions},
  KeychainKind, Wallet,
};

fn generate_key() -> Result<impl DerivableKey<Segwitv0> + Clone> {
  let password = Some("password".to_string());

  let mnemonic = Mnemonic::generate((WordCount::Words12, Language::English))
    .map_err(|err| err.expect("Failed to generate mnemonic"))?;

  Ok((mnemonic, password))
}

pub(crate) fn run(options: Options) -> Result {
  println!("[~] Setting up ordinal wallet...");

  let key = generate_key()?;

  let wallet = Wallet::new(
    Bip84(key.clone(), KeychainKind::External),
    Some(Bip84(key.clone(), KeychainKind::Internal)),
    Network::Regtest,
    SqliteDatabase::new("foo".into()),
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
        Bip84(key.clone(), KeychainKind::External),
        Some(Bip84(key, KeychainKind::Internal)),
        Network::Regtest,
        &Secp256k1::new(),
      )?,
      skip_blocks: None,
    })?,
    SyncOptions::default(),
  )?;

  println!("Setup complete.");

  Ok(())
}
