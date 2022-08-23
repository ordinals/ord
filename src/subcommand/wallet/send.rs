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

    let utxo = purse.find(&options, self.ordinal)?;

    let ordinals = purse.special_ordinals(&options, utxo.outpoint)?;

    if !ordinals.is_empty() && (ordinals.len() > 1 || ordinals[0] != self.ordinal) {
      bail!(
        "Trying to send ordinal {} but UTXO also contains ordinal(s) {}",
        self.ordinal,
        ordinals
          .iter()
          .map(|ordinal| format!("{ordinal} ({})", ordinal.rarity()))
          .collect::<Vec<String>>()
          .join(", ")
      );
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
