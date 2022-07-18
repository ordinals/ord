use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!("[~] Setting up ordinal wallet...");

  let key = get_key()?;

  let wallet = Wallet::new(
    Bip84(key.clone(), KeychainKind::External),
    None,
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
        None,
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
