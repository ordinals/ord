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

    let index = Index::index(&options)?;

    let utxo = purse.find(&index, self.ordinal)?;

    let ordinals = purse.special_ordinals(&index, utxo.outpoint)?;

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

    fn iter_funding_utxos(
      psbt: &PartiallySignedTransaction,
    ) -> impl Iterator<Item = Result<&TxOut>> {
      assert_eq!(psbt.inputs.len(), psbt.unsigned_tx.input.len());

      psbt
        .unsigned_tx
        .input
        .iter()
        .zip(&psbt.inputs)
        .map(|(tx_input, psbt_input)| {
          match (&psbt_input.witness_utxo, &psbt_input.non_witness_utxo) {
            (Some(witness_utxo), _) => Ok(witness_utxo),
            (None, Some(non_witness_utxo)) => {
              let vout = tx_input.previous_output.vout as usize;
              non_witness_utxo
                .output
                .get(vout)
                .context("PSBT UTXO out of bounds")
            }
            (None, None) => Err(anyhow!("Missing UTXO")),
          }
        })
    }

    let input_value = iter_funding_utxos(&psbt)
      .map(|result| result.map(|utxo| utxo.value))
      .sum::<Result<u64>>()?;

    let output_value = psbt
      .unsigned_tx
      .output
      .iter()
      .map(|output| output.value)
      .sum::<u64>();

    let mut offset = 0;

    for (start, end) in Purse::list_unspent(&index, utxo.outpoint)? {
      if start <= self.ordinal.n() && self.ordinal.n() < end {
        offset += self.ordinal.n() - start;
        break;
      } else {
        offset += end - start;
      }
    }

    if offset >= output_value {
      bail!(
        "Ordinal {} is {} sat away from the end of the output which is within the {} sat fee range",
        self.ordinal,
        input_value - offset,
        input_value - output_value
      );
    }

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
