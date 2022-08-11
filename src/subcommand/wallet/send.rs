use {super::*, bdk::wallet::signer::SignOptions, bdk::LocalUtxo, bitcoin::Address};

// ord wallet send --address <address> --ordinal <ordinal>

#[derive(Debug, Parser)]
pub(crate) struct Send {
  #[clap(long)]
  address: Address,
  #[clap(long)]
  ordinal: Ordinal,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::index(&options)?;

    let wallet = get_wallet(options)?;

    let utxo = self.find(index, &wallet)?;

    let (mut psbt, _details) = {
      let mut builder = wallet.build_tx();

      builder
        .add_utxo(utxo.outpoint)?
        .add_recipient(self.address.script_pubkey(), utxo.txout.value);

      builder.finish()?
    };

    if !wallet.sign(&mut psbt, SignOptions::default())? {
      bail!("Failed to sign transaction.")
    }

    Ok(())
  }

  fn find(&self, index: Index, wallet: &bdk::wallet::Wallet<SqliteDatabase>) -> Result<LocalUtxo> {
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
