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
    let wallet = Purse::load(&options)?;

    let utxo = wallet.find(&options, self.ordinal)?;

    let ordinals = wallet.special_ordinals(&options, utxo.outpoint)?;

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
      let mut builder = wallet.wallet.build_tx();

      builder
        .manually_selected_only()
        .fee_rate(FeeRate::from_sat_per_vb(2.0))
        .add_utxo(utxo.outpoint)?
        .drain_to(self.address.script_pubkey());

      builder.finish()?
    };

    if !wallet.wallet.sign(&mut psbt, SignOptions::default())? {
      bail!("Failed to sign transaction.");
    }

    let tx = psbt.extract_tx();

    wallet.blockchain.broadcast(&tx)?;

    println!(
      "Sent ordinal {} to address {}: {}",
      self.ordinal.0,
      self.address,
      tx.txid()
    );

    Ok(())
  }
}
