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

    let output_value = psbt
      .unsigned_tx
      .output
      .iter()
      .map(|output| output.value)
      .sum::<u64>();

    let list = index.list(utxo.outpoint)?;

    let mut offset = 0;

    match list {
      Some(List::Unspent(ranges)) => {
        for (start, end) in ranges {
          if start <= self.ordinal.n() && self.ordinal.n() < end {
            offset += self.ordinal.n() - start;
            break;
          } else {
            offset += end - start;
          }
        }
      }
      Some(List::Spent(txid)) => {
        todo!()
      }
      None => todo!(),
    }

    if offset >= output_value {
      bail!("Trying to send ordinal that would have been used in fees");
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
