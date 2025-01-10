use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
}

#[derive(Debug, Parser)]
pub(crate) struct Accept {
  #[arg(long, help = "Accept <PSBT>")]
  psbt: String,
  #[arg(long)]
  inscription: InscriptionId,
  #[arg(long)]
  amount: Amount,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let psbt = base64_decode(&self.psbt).context("failed to base64 decode PSBT")?;

    let psbt = Psbt::deserialize(&psbt).context("failed to deserialize PSBT")?;

    let mut outgoing = BTreeSet::new();

    for input in &psbt.unsigned_tx.input {
      if wallet.utxos().contains_key(&input.previous_output) {
        outgoing.insert(input.previous_output);
      }
    }

    ensure! {
      outgoing.len() <= 1,
      "PSBT contains {} inputs owned by wallet", outgoing.len(),
    }

    let Some(outgoing) = outgoing.into_iter().next() else {
      bail!("PSBT contains no inputs owned by wallet");
    };

    if let Some(runes) = wallet.get_runes_balances_in_output(&outgoing)? {
      ensure! {
        runes.is_empty(),
        "outgoing input {} contains runes", outgoing,
      }
    }

    let Some(inscriptions) = wallet.get_inscriptions_in_output(&outgoing) else {
      bail! {
        "index must have inscription index to accept PSBT",
      }
    };

    ensure! {
      inscriptions.len() <= 1,
      "outgoing input {} contains {} inscriptions", outgoing, inscriptions.len(),
    }

    let Some(inscription) = inscriptions.into_iter().next() else {
      bail!("outgoing input contains no inscriptions");
    };

    ensure! {
      inscription == self.inscription,
      "unexpected outgoing inscription {inscription}",
    }

    let balance_change = wallet.simulate_transaction(&psbt.unsigned_tx)?;

    ensure! {
      balance_change == self.amount.to_signed()?,
      "unexpected simulated balance change of {balance_change}",
    }

    let psbt = wallet
      .bitcoin_client()
      .wallet_process_psbt(&base64_encode(&psbt.serialize()), Some(true), None, None)?
      .psbt;

    let finalized = wallet.bitcoin_client().finalize_psbt(&psbt, None)?;

    let signed_tx = finalized
      .hex
      .ok_or_else(|| anyhow!("unable to sign transaction"))?;

    let txid = wallet.send_raw_transaction(&signed_tx, None)?;

    Ok(Some(Box::new(Output { txid })))
  }
}
