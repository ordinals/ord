use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Send {
  #[clap(long)]
  address: Address,
  #[clap(long)]
  ordinal: Ordinal,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let purse = Purse::load(&options)?;

    eprintln!("{:?}", purse.wallet.list_unspent()?);

    let utxo = purse.find(&options, self.ordinal)?;

    let ordinals = purse.special_ordinals(&options, utxo.outpoint)?;

    if !ordinals.is_empty() {
      match ordinals.len() {
        1 => {
          if ordinals[0] != self.ordinal {
            bail!(
              "UTXO contains a single uncommon or better ordinal that does not match the ordinal you are trying to send."
            )
          }
        }
        _ => bail!("UTXO contains more than one uncommon or better ordinal."),
      }
    }

    let (mut psbt, _details) = {
      let mut builder = purse.wallet.build_tx();

      builder
        .manually_selected_only()
        .fee_rate(FeeRate::from_sat_per_vb(2.0))
        .add_utxo(utxo.outpoint)?
        .drain_to(self.address.script_pubkey());

      builder.finish()?
    };

    if !purse.wallet.sign(&mut psbt, SignOptions::default())? {
      bail!("Failed to sign transaction.");
    }

    let tx = psbt.extract_tx();

    purse.blockchain.broadcast(&tx)?;

    println!(
      "Sent ordinal {} to address {}: {}",
      self.ordinal.0,
      self.address,
      tx.txid()
    );

    Ok(())
  }
}
