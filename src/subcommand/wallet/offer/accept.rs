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

#[derive(Deserialize)]
struct SimulateRawTransactionResult {
  #[serde(with = "bitcoin::amount::serde::as_btc")]
  balance_change: Amount,
}

#[derive(Serialize)]
struct SimulateRawTransactionOptions {
  include_watchonly: bool,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let psbt = Psbt::deserialize(&base64_decode(&self.psbt)?)?;

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

    let unsigned_tx_hex = {
      let mut buffer = Vec::new();
      psbt.unsigned_tx.consensus_encode(&mut buffer).unwrap();
      hex::encode(buffer)
    };

    let simulation = wallet
      .bitcoin_client()
      .call::<SimulateRawTransactionResult>(
        "simulaterawtransaction",
        &[
          [unsigned_tx_hex].into(),
          serde_json::to_value(SimulateRawTransactionOptions {
            include_watchonly: false,
          })
          .unwrap(),
        ],
      )?;

    ensure! {
      simulation.balance_change == self.amount,
      "unexpected simulated balance change of {}",
      simulation.balance_change,
    }

    let psbt = wallet
      .bitcoin_client()
      .wallet_process_psbt(&base64_encode(psbt.serialize()), Some(true), None, None)?
      .psbt;

    let signed_tx = wallet
      .bitcoin_client()
      .finalize_psbt(&psbt, None)?
      .hex
      .ok_or_else(|| anyhow!("unable to sign transaction"))?;

    let txid = wallet.send_raw_transaction(&signed_tx, None)?;

    Ok(Some(Box::new(Output { txid })))
  }
}
