use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
  pub psbt: String,
  pub fee: u64,
}

#[derive(Debug, Parser)]
pub(crate) struct Accept {
  #[arg(long, help = "Accept <PSBT> offer")]
  psbt: String,
  #[arg(
    long,
    help = "Assert offer is for <INSCRIPTION> or at least <DECIMAL:RUNE>"
  )]
  outgoing: Outgoing,
  #[arg(long, help = "Assert offer requires at most <AMOUNT>")]
  amount: Amount,
  #[arg(
    long,
    help = "Include <AMOUNT> postage with receive output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
  #[arg(long, help = "<FEE_RATE> for finalized transaction.")]
  fee_rate: FeeRate,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    match self.outgoing {
      Outgoing::InscriptionId(inscription_id) => {
        self.accept_inscription_sell_offer(wallet, inscription_id)
      }
      Outgoing::Rune { decimal, rune } => self.accept_rune_sell_offer(wallet, decimal, rune),
      _ => bail!("outgoing must be either <INSCRIPTION> or <DECIMAL:RUNE>"),
    }
  }

  fn accept_inscription_sell_offer(
    &self,
    _wallet: Wallet,
    _inscription_id: InscriptionId,
  ) -> SubcommandResult {
    bail!("inscription sell offers not yet implemented");
  }

  fn accept_rune_sell_offer(
    &self,
    wallet: Wallet,
    decimal: Decimal,
    spaced_rune: SpacedRune,
  ) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "creating runes offer with `ord offer` requires index created with `--index-runes` flag",
    );

    let psbt = base64_decode(&self.psbt).context("failed to base64 decode PSBT")?;

    let psbt = Psbt::deserialize(&psbt).context("failed to deserialize PSBT")?;

    let mut input_rune_balance = 0;
    let mut input_sat_value = Amount::from_sat(0);
    let mut output_sat_value = Amount::from_sat(0);

    // get input sats and input rune balance of PSBT offer
    for input in &psbt.unsigned_tx.input {
      if let Some(output_info) = wallet.get_output_info(input.previous_output)? {
        if let Some(runes) = output_info.runes {
          if let Some(pile) = runes.get(&spaced_rune) {
            input_rune_balance += pile.amount;
          }
        }
        input_sat_value += Amount::from_sat(output_info.value);
      }
    }

    ensure! {
      input_rune_balance >= decimal.value,
      "PSBT contains {} {} runes in input(s) but {} {} required",
      input_rune_balance,
      spaced_rune,
      decimal.value,
      spaced_rune,
    }

    // get output sats of PSBT offer
    for output in &psbt.unsigned_tx.output {
      output_sat_value += output.value;
    }

    ensure! {
      input_sat_value + self.amount >= output_sat_value,
      "PSBT requires more sats than user allows ({} > {})",
      output_sat_value - input_sat_value,
      self.amount,
    }

    let postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let receive_output = TxOut {
      value: postage,
      script_pubkey: wallet.get_change_address()?.into(),
    };

    let mut change_output = TxOut {
      value: Amount::ZERO,
      script_pubkey: wallet.get_change_address()?.into(),
    };

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    wallet.lock_non_cardinal_outputs()?;

    // get cardinal utxos sorted by value
    let mut unlocked_sorted_utxos = wallet
      .bitcoin_client()
      .list_unspent(None, None, None, None, None)?
      .into_iter()
      .map(|utxo| {
        let outpoint = OutPoint::new(utxo.txid, utxo.vout);
        let txout = TxOut {
          script_pubkey: utxo.script_pub_key,
          value: utxo.amount,
        };
        (outpoint, txout)
      })
      .collect::<Vec<(OutPoint, TxOut)>>();

    unlocked_sorted_utxos.sort_by_key(|(_, txout)| std::cmp::Reverse(txout.value));

    let mut remaining_amount_to_fund = postage + output_sat_value - input_sat_value;
    let mut next_utxo = 0;

    // insert inputs until funding amount is satisfied
    for (outpoint, txout) in &unlocked_sorted_utxos {
      inputs.push(TxIn {
        previous_output: *outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      });
      next_utxo += 1;

      if txout.value >= remaining_amount_to_fund {
        // add residual amount to postage
        change_output.value += txout.value - remaining_amount_to_fund;
        remaining_amount_to_fund = Amount::ZERO;
        break;
      } else {
        remaining_amount_to_fund -= txout.value;
      }
    }

    ensure! {
      remaining_amount_to_fund == Amount::ZERO,
      "Insufficient funds to purchase PSBT offer (requires additional {})",
      remaining_amount_to_fund,
    }

    // insert inputs in PSBT offer, starting at 1th-index
    inputs.splice(1..1, psbt.clone().unsigned_tx.input);

    // insert the postage/change output first, followed by outputs in PSBT offer
    outputs.push(receive_output.clone());
    outputs.extend(psbt.clone().unsigned_tx.output);
    outputs.push(change_output.clone());

    let mut unsigned_tx = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: inputs,
      output: outputs,
    };

    // deduct fee from first output or add necessary inputs to meet desired fee rate
    loop {
      // calculate fee using vsize of fully signed transaction
      let signed_tx = self.get_signed_tx(&wallet, &psbt, &unsigned_tx)?;
      let desired_fee = self.fee_rate.fee(signed_tx.vsize());

      // reduce output value by desired fee, if remainder satisfies postage
      let change_output = unsigned_tx.output.last_mut().unwrap();
      if change_output.value >= desired_fee {
        change_output.value -= desired_fee;
        break;
      }

      ensure! {
        next_utxo < unlocked_sorted_utxos.len(),
        "Insufficient funds to meet desired fee rate (at least {} sats required)",
        desired_fee,
      }

      // insert the next utxo and add sat value to the first output
      let (outpoint, txout) = &unlocked_sorted_utxos[next_utxo];
      next_utxo += 1;

      unsigned_tx.input.push(TxIn {
        previous_output: *outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      });

      change_output.value += txout.value;
    }

    // remove change output if dust
    let change_output = unsigned_tx.output.last().unwrap();
    if change_output.value < change_output.script_pubkey.minimal_non_dust() {
      unsigned_tx.output.pop();
    }

    let (txid, psbt, fee) =
      wallet.sign_and_broadcast_transaction(unsigned_tx, self.dry_run, None)?;

    Ok(Some(Box::new(Output { txid, psbt, fee })))
  }

  // returns signed tx given psbt offer and full unsigned transaction
  fn get_signed_tx(
    &self,
    wallet: &Wallet,
    psbt: &Psbt,
    unsigned_tx: &Transaction,
  ) -> Result<Transaction> {
    let mut unsigned_psbt = Psbt::from_unsigned_tx(unsigned_tx.clone())?;
    unsigned_psbt.inputs.splice(1.., psbt.inputs.clone());

    let result = wallet.bitcoin_client().call::<String>(
      "utxoupdatepsbt",
      &[base64_encode(&unsigned_psbt.serialize()).into()],
    )?;

    let result = wallet
      .bitcoin_client()
      .wallet_process_psbt(&result, Some(true), None, None)?;

    ensure! {
      result.complete,
      "At least 1 PSBT input is unsigned and cannot be signed by wallet"
    }

    let signed_psbt = base64_decode(&result.psbt).context("failed to base64 decode PSBT")?;

    let signed_psbt = Psbt::deserialize(&signed_psbt).context("failed to deserialize PSBT")?;

    let signed_tx = signed_psbt.extract_tx()?;

    Ok(signed_tx)
  }
}
