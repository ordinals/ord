use {
  super::*, bdk::blockchain::Blockchain, bdk::blockchain::RpcBlockchain,
  bdk::wallet::signer::SignOptions, bdk::LocalUtxo, bitcoin::Address,
};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  #[clap(long)]
  address: Address,
  #[clap(long)]
  ordinal: Ordinal,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let wallet = get_wallet(options.clone())?;

    let utxo = self.find(options.clone(), &wallet)?;

    let (mut psbt, _details) = {
      let mut builder = wallet.build_tx();

      builder
        .manually_selected_only()
        .fee_absolute(0)
        .allow_dust(true)
        .add_utxo(utxo.outpoint)?
        .add_recipient(self.address.script_pubkey(), utxo.txout.value);

      builder.finish()?
    };

    if !wallet.sign(&mut psbt, SignOptions::default())? {
      bail!("Failed to sign transaction.");
    }

    let path = data_dir()
      .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
      .join("ord");

    if !path.exists() {
      return Err(anyhow!("Wallet doesn't exist."));
    }

    let blockchain = RpcBlockchain::from_config(&RpcConfig {
      url: options.rpc_url(),
      auth: Auth::Cookie {
        file: options.cookie_file()?,
      },
      network: options.network,
      wallet_name: wallet_name_from_descriptor(
        Bip84(
          (
            Mnemonic::from_entropy(&fs::read(path.join("entropy"))?)?,
            None,
          ),
          KeychainKind::External,
        ),
        None,
        options.network,
        &Secp256k1::new(),
      )?,
      skip_blocks: None,
    })?;

    let tx = psbt.extract_tx();

    blockchain.broadcast(&tx)?;

    println!(
      "Sent ordinal {} to address {}, {}",
      self.ordinal.0,
      self.address,
      tx.txid()
    );

    Ok(())
  }

  fn find(
    &self,
    options: Options,
    wallet: &bdk::wallet::Wallet<SqliteDatabase>,
  ) -> Result<LocalUtxo> {
    let index = Index::index(&options)?;

    for utxo in wallet.list_unspent()? {
      if let Some(ranges) = index.list(utxo.outpoint)? {
        for (start, end) in ranges {
          if start <= self.ordinal.0 && self.ordinal.0 < end {
            return Ok(utxo);
          }
        }
      }
    }

    bail!("No utxo found that contains ordinal {}.", self.ordinal);
  }
}
