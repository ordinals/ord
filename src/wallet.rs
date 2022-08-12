use super::*;

#[derive(Debug)]
pub(crate) struct Wallet {
  pub(crate) wallet: bdk::wallet::Wallet<SqliteDatabase>,
  pub(crate) blockchain: RpcBlockchain,
}

impl Wallet {
  pub(crate) fn setup(options: &Options) -> Result {
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

    Ok(())
  }

  pub(crate) fn load(options: &Options) -> Result<Self> {
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

    let blockchain = RpcBlockchain::from_config(&RpcConfig {
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
    })?;

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(Self { wallet, blockchain })
  }

  pub(crate) fn find(&self, options: &Options, ordinal: Ordinal) -> Result<LocalUtxo> {
    let index = Index::index(options)?;

    for utxo in self.wallet.list_unspent()? {
      if let Some(ranges) = index.list(utxo.outpoint)? {
        for (start, end) in ranges {
          if start <= ordinal.0 && ordinal.0 < end {
            return Ok(utxo);
          }
        }
      }
    }

    bail!("No utxo found that contains ordinal {}.", ordinal);
  }
}
