use super::*;

use bdk::{
  blockchain::{
    rpc::{Auth, RpcBlockchain, RpcConfig},
    ConfigurableBlockchain,
  },
  database::memory::MemoryDatabase,
  keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, GeneratableKey, GeneratedKey,
  },
  miniscript::miniscript::Segwitv0,
  template::Bip84,
  wallet::{wallet_name_from_descriptor, SyncOptions},
  KeychainKind, Wallet,
};

fn generate_key() -> Result<impl DerivableKey<Segwitv0> + Clone> {
  let password = Some("password".to_string());

  let mnemonic: GeneratedKey<_, _> = Mnemonic::generate((WordCount::Words12, Language::English))
    .map_err(|e| e.expect("Unknown Error"))?;

  Ok((mnemonic, password))
}

pub(crate) fn run(options: Options) -> Result {
  println!("[~] Setting up ordinal wallet...");

  let key = generate_key()?;

  let wallet = Wallet::new(
    Bip84(key.clone(), KeychainKind::External),
    Some(Bip84(key.clone(), KeychainKind::Internal)),
    Network::Signet,
    MemoryDatabase::new(),
  )?;

  wallet.sync(
    &RpcBlockchain::from_config(&RpcConfig {
      url: options.rpc_url.unwrap(),
      auth: Auth::Cookie {
        file: options.cookie_file.unwrap(),
      },
      network: Network::Signet,
      wallet_name: wallet_name_from_descriptor(
        Bip84(key.clone(), KeychainKind::External),
        Some(Bip84(key.clone(), KeychainKind::Internal)),
        Network::Signet,
        &Secp256k1::new(),
      )?,
      skip_blocks: None,
    })?,
    SyncOptions::default(),
  )?;

  println!("Setup complete.");

  Ok(())
}
