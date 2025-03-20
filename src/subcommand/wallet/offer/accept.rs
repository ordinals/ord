use super::*;

#[derive(PartialEq)]
enum Signature<'a> {
  Script(&'a Script),
  Witness(&'a Witness),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
}

#[derive(Debug, Parser)]
pub(crate) struct Accept {
  #[arg(long, help = "Assert offer is for <AMOUNT>")]
  amount: Amount,
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
  #[arg(long, help = "Assert offer is for <INSCRIPTION>")]
  inscription: Option<InscriptionId>,
  #[arg(long, help = "<DECIMAL:RUNE> to make offer for.")]
  rune: Option<Outgoing>,
  #[arg(long, help = "Accept <PSBT> offer")]
  psbt: String,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
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

    match (self.inscription, self.rune.clone()) {
      (Some(inscription), None) => {
        self.check_inscription_buy_offer(&wallet, outgoing, inscription)?
      }
      (None, Some(rune)) => {
        self.check_rune_buy_offer(&wallet, psbt.unsigned_tx.clone(), outgoing, rune)?
      }
      (None, None) => bail!("must include either --inscription or --rune"),
      (Some(_), Some(_)) => bail!("cannot include both --inscription and --rune"),
    }

    let balance_change = wallet.simulate_transaction(&psbt.unsigned_tx)?;

    ensure! {
      balance_change == self.amount.to_signed()?,
      "unexpected balance change of {balance_change}",
    }

    let signatures = Self::psbt_signatures(&psbt)?;

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

    let txid = if self.dry_run {
      psbt.unsigned_tx.compute_txid()
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
          .zip(Self::tx_signatures(&signed_tx)?)
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

      wallet.send_raw_transaction(&signed_tx, None)?
    };

    Ok(Some(Box::new(Output { txid })))
  }

  fn check_inscription_buy_offer(
    &self,
    wallet: &Wallet,
    outgoing: OutPoint,
    inscription_id: InscriptionId,
  ) -> Result {
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

    Ok(())
  }

  fn check_rune_buy_offer(
    &self,
    wallet: &Wallet,
    unsigned_tx: Transaction,
    outgoing: OutPoint,
    rune: Outgoing,
  ) -> Result {
    let (decimal, spaced_rune) = match rune {
      Outgoing::Rune { decimal, rune } => (decimal, rune),
      _ => bail!("invalid format for --rune (must be `DECIMAL:RUNE`)"),
    };

    ensure!(
      wallet.has_rune_index(),
      "accepting rune offer with `offer` requires index created with `--index-runes` flag",
    );

    let (id, _, _) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let Some(runes) = wallet.get_runes_balances_in_output(&outgoing)? else {
      bail!("outgoing input contains no runes");
    };

    if let Some(inscriptions) = wallet.get_inscriptions_in_output(&outgoing)? {
      ensure! {
        inscriptions.is_empty(),
        "outgoing input {} contains {} inscription(s)",
        outgoing,
        inscriptions.len()
      }
    };

    let Some(pile) = runes.get(&spaced_rune) else {
      bail!(format!(
        "outgoing input {} does not contain rune {}",
        outgoing, spaced_rune
      ));
    };

    ensure! {
      pile.amount == decimal.value,
      "unexpected rune {} balance at outgoing input {} ({} vs. {})",
      spaced_rune,
      outgoing,
      pile.amount,
      decimal.value
    }

    if runes.len() > 1 {
      let Some(runestone) = Runestone::decipher(&unsigned_tx) else {
        bail!("missing runestone in PSBT");
      };

      let expected_runestone = Runestone {
        edicts: vec![Edict {
          amount: 0,
          id,
          output: 2,
        }],
        ..default()
      };

      ensure! {
        runestone == Artifact::Runestone(expected_runestone),
        "unexpected runestone in PSBT"
      }

      ensure! {
        !unsigned_tx.output.is_empty() &&
        unsigned_tx.output[0].script_pubkey == wallet.utxos().get(&outgoing).unwrap().script_pubkey,
        "unexpected seller address in PSBT"
      }
    } else {
      ensure! {
        Runestone::decipher(&unsigned_tx).is_none(),
        "unexpected runestone in PSBT"
      }
    }

    Ok(())
  }

  fn psbt_signatures(psbt: &Psbt) -> Result<Vec<Option<Signature>>> {
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
}
