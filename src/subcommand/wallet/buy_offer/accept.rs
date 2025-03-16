use super::*;

#[derive(PartialEq)]
pub enum Signature<'a> {
  Script(&'a Script),
  Witness(&'a Witness),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
  pub psbt: String,
  pub fee: u64,
}

#[derive(Debug, Parser)]
pub(crate) struct Accept {
  #[arg(long, help = "Assert offer is for <AMOUNT>")]
  pub amount: Amount,
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub dry_run: bool,
  #[arg(long, help = "Assert offer is for <INSCRIPTION> or <DECIMAL:RUNE>")]
  pub outgoing: Outgoing,
  #[arg(long, help = "Accept <PSBT> offer")]
  pub psbt: String,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let (txid, psbt, fee) = match self.outgoing {
      Outgoing::InscriptionId(inscription_id) => {
        self.accept_inscription_buy_offer(wallet, inscription_id)?
      }
      Outgoing::Rune { decimal, rune } => self.accept_rune_buy_offer(wallet, decimal, rune)?,
      _ => bail!("outgoing must be either <INSCRIPTION> or <DECIMAL:RUNE>"),
    };

    Ok(Some(Box::new(Output {
      txid,
      psbt,
      fee: fee.to_sat(),
    })))
  }

  pub fn accept_inscription_buy_offer(
    &self,
    wallet: Wallet,
    inscription_id: InscriptionId,
  ) -> Result<(Txid, String, Amount)> {
    let psbt = base64_decode(&self.psbt).context("failed to base64 decode PSBT")?;

    let psbt = Psbt::deserialize(&psbt).context("failed to deserialize PSBT")?;

    let mut outgoing = BTreeMap::new();

    for (index, input) in psbt.unsigned_tx.input.iter().enumerate() {
      if wallet.utxos().contains_key(&input.previous_output) {
        outgoing.insert(index, input.previous_output);
      }
    }

    ensure! {
      outgoing.len() <= 1,
      "PSBT contains {} inputs owned by wallet", outgoing.len(),
    }

    let Some((index, outgoing)) = outgoing.into_iter().next() else {
      bail!("PSBT contains no inputs owned by wallet");
    };

    if let Some(runes) = wallet.get_runes_balances_in_output(&outgoing)? {
      ensure! {
        runes.is_empty(),
        "outgoing input {} contains runes", outgoing,
      }
    }

    let Some(inscriptions) = wallet.get_inscriptions_in_output(&outgoing)? else {
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
      inscription == inscription_id,
      "unexpected outgoing inscription {inscription}",
    }

    let balance_change = wallet.simulate_transaction(&psbt.unsigned_tx)?;

    ensure! {
      balance_change == self.amount.to_signed()?,
      "unexpected balance change of {balance_change}",
    }

    let signatures = psbt_signatures(&psbt)?;

    for (i, signature) in signatures.iter().enumerate() {
      let outpoint = psbt.unsigned_tx.input[i].previous_output;

      if i == index {
        ensure! {
          signature.is_none(),
          "seller input `{outpoint}` is signed: seller input must not be signed",
        }
      } else {
        ensure! {
          signature.is_some(),
          "buyer input `{outpoint}` is unsigned: buyer inputs must be signed",
        }
      }
    }

    let result = if self.dry_run {
      (
        psbt.unsigned_tx.compute_txid(),
        base64_encode(&psbt.serialize()),
        psbt.fee()?,
      )
    } else {
      let signed_psbt = wallet
        .bitcoin_client()
        .wallet_process_psbt(&base64_encode(&psbt.serialize()), Some(true), None, None)?
        .psbt;

      let signed_tx = wallet
        .bitcoin_client()
        .finalize_psbt(&signed_psbt, None)?
        .hex
        .ok_or_else(|| anyhow!("unable to sign transaction"))?;

      {
        let signed_tx = Transaction::consensus_decode(&mut signed_tx.as_slice())
          .context("unable to decode finalized transaction")?;

        ensure! {
          signed_tx.input.len() == psbt.inputs.len() &&
          signed_tx.input.len() == psbt.unsigned_tx.input.len(),
          "signed transaction input length mismatch",
        }

        for (i, (old, new)) in signatures
          .into_iter()
          .zip(tx_signatures(&signed_tx)?)
          .enumerate()
        {
          let outpoint = signed_tx.input[i].previous_output;

          if i == index {
            ensure! {
              new.is_some(),
              "seller input `{outpoint}` was not signed by wallet",
            }
          } else {
            ensure! {
              old == new,
              "buyer input `{outpoint}` signature changed after signing",
            }
          }
        }
      }

      (
        wallet.send_raw_transaction(&signed_tx, None)?,
        signed_psbt,
        psbt.fee()?,
      )
    };

    Ok(result)
  }

  fn accept_rune_buy_offer(
    &self,
    _wallet: Wallet,
    _decimal: Decimal,
    _spaced_rune: SpacedRune,
  ) -> Result<(Txid, String, Amount)> {
    bail!("rune buy offers not yet implemented");
  }
}

pub fn psbt_signatures(psbt: &Psbt) -> Result<Vec<Option<Signature>>> {
  psbt
    .inputs
    .iter()
    .map(
      |input| match (&input.final_script_sig, &input.final_script_witness) {
        (None, None) => Ok(None),
        (Some(script), None) => Ok(Some(Signature::Script(script))),
        (None, Some(witness)) => Ok(Some(Signature::Witness(witness))),
        (Some(_), Some(_)) => bail!("input contains both scriptsig and witness"),
      },
    )
    .collect()
}

fn tx_signatures(tx: &Transaction) -> Result<Vec<Option<Signature>>> {
  tx.input
    .iter()
    .map(|input| {
      match (
        (!input.script_sig.is_empty()).then_some(&input.script_sig),
        (!input.witness.is_empty()).then_some(&input.witness),
      ) {
        (None, None) => Ok(None),
        (Some(script), None) => Ok(Some(Signature::Script(script))),
        (None, Some(witness)) => Ok(Some(Signature::Witness(witness))),
        (Some(_), Some(_)) => bail!("input contains both scriptsig and witness"),
      }
    })
    .collect()
}
