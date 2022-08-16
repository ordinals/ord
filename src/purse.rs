use super::*;

use bdk::keys::DerivableKey;
use bdk::miniscript::miniscript::Segwitv0;

#[derive(Debug)]
pub(crate) struct Purse {
  pub(crate) blockchain: RpcBlockchain,
  pub(crate) options: Options,
  pub(crate) wallet: bdk::wallet::Wallet<SqliteDatabase>,
}

impl Purse {
  pub(crate) fn init(options: &Options) -> Result {
    let path = data_dir()
      .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
      .join("ord");

    if path.exists() {
      return Err(anyhow!("Wallet already exists."));
    }

    fs::create_dir_all(&path)?;

    let seed = Mnemonic::generate_in_with(&mut rand::thread_rng(), Language::English, 12)?;

    fs::write(path.join("entropy"), seed.to_entropy())?;

    let wallet = bdk::wallet::Wallet::new(
      Bip84((seed.clone(), None), KeychainKind::External),
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
      &Self::blockchain(options, (seed, None))?,
      SyncOptions::default(),
    )?;

    eprintln!("Wallet initialized.");

    Ok(())
  }

  pub(crate) fn load(options: Options) -> Result<Self> {
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

    let blockchain = Self::blockchain(&options, key)?;

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(Self {
      blockchain,
      options,
      wallet,
    })
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<LocalUtxo> {
    let index = Index::index(&self.options)?;

    for utxo in self.wallet.list_unspent()? {
      if let Some(ranges) = index.list(utxo.outpoint)? {
        for (start, end) in ranges {
          if ordinal.0 >= start && ordinal.0 < end {
            return Ok(utxo);
          }
        }
      }
    }

    bail!("No utxo contains {}˚.", ordinal);
  }

  fn blockchain(
    options: &Options,
    key: impl DerivableKey<Segwitv0> + Clone,
  ) -> Result<RpcBlockchain> {
    Ok(RpcBlockchain::from_config(&RpcConfig {
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
    })?)
  }
}
