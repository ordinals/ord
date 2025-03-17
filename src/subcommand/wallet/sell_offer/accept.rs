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
    let psbt = base64_decode(&self.psbt).context("failed to base64 decode PSBT")?;

    let psbt = Psbt::deserialize(&psbt).context("failed to deserialize PSBT")?;

    ensure! {
      psbt.inputs.len() == psbt.outputs.len() &&
      psbt.unsigned_tx.input.len() == psbt.unsigned_tx.output.len(),
      "PSBT must contain the same number of inputs and outputs",
    }

    ensure! {
      psbt.unsigned_tx.input.len() == psbt.inputs.len(),
      "PSBT input length mismatch",
    }

    ensure! {
      buy_offer::accept::psbt_signatures(&psbt)?.into_iter().flatten().count() == psbt.inputs.len(),
      "PSBT must be fully signed",
    }

    let (txid, psbt, fee) = match self.outgoing {
      Outgoing::Rune { decimal, rune } => {
        self.accept_rune_sell_offer(wallet, psbt, decimal, rune)?
      }
      Outgoing::InscriptionId(_) => bail!("inscription sell offers not yet implemented"),
      _ => bail!("outgoing must be either <INSCRIPTION> or <DECIMAL:RUNE>"),
    };

    Ok(Some(Box::new(Output {
      txid,
      psbt,
      fee: fee.to_sat(),
    })))
  }

  fn accept_rune_sell_offer(
    &self,
    wallet: Wallet,
    psbt: Psbt,
    decimal: Decimal,
    spaced_rune: SpacedRune,
  ) -> Result<(Txid, String, Amount)> {
    ensure!(
      wallet.has_rune_index(),
      "creating runes offer with `ord offer` requires index created with `--index-runes` flag",
    );

    let mut input_rune_balance = 0;
    let mut input_sat_value = Amount::from_sat(0);
    let mut output_sat_value = Amount::from_sat(0);

    // get input sats and input rune balance of PSBT offer
    for input in &psbt.unsigned_tx.input {
      ensure! {
        wallet.output_exists(input.previous_output)?,
        "PSBT spends utxo {} that does not exist",
        input.previous_output
      }

      if let Some(output_info) = wallet.get_any_output_info(input.previous_output)? {
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

    let mut remaining_amount_to_fund = if postage + output_sat_value > input_sat_value {
      postage + output_sat_value - input_sat_value
    } else {
      Amount::ZERO
    };

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
      let (signed_psbt, _) = self.process_psbt(&wallet, &psbt, &unsigned_tx, true)?;
      let signed_tx = signed_psbt.extract_tx()?;
      let desired_fee = self.fee_rate.fee(signed_tx.vsize());

      // reduce output value by desired fee, if remainder satisfies postage
      let change_output = unsigned_tx.output.last_mut().unwrap();
      if change_output.value >= desired_fee {
        change_output.value -= desired_fee;
        break;
      }

      ensure! {
        next_utxo < unlocked_sorted_utxos.len(),
        "Insufficient funds to meet desired fee rate (at least {} required)",
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

    let last_index = unsigned_tx.output.len() - 1;
    let change_value = unsigned_tx.output[last_index].value;
    let minimal_dust = unsigned_tx.output[last_index]
      .script_pubkey
      .minimal_non_dust();

    // remove change output if dust and add dust value to receive output
    if change_value < minimal_dust {
      let size = consensus::encode::serialize(&unsigned_tx.output[last_index]).len();
      let fee_saving = self.fee_rate.fee(size) - Amount::from_sat(1); // deduct 1 sat to avoid overshooting
      unsigned_tx.output[0].value += change_value + fee_saving;
      unsigned_tx.output.pop();
    }

    let result = if self.dry_run {
      let (psbt, encoded_psbt) = self.process_psbt(&wallet, &psbt, &unsigned_tx, false)?;

      (unsigned_tx.compute_txid(), encoded_psbt, psbt.fee()?)
    } else {
      let (signed_psbt, encoded_psbt) = self.process_psbt(&wallet, &psbt, &unsigned_tx, true)?;
      let fee = signed_psbt.fee()?;
      let signed_tx = signed_psbt.extract_tx()?;

      (
        wallet.send_raw_transaction(&consensus::encode::serialize(&signed_tx), None)?,
        encoded_psbt,
        fee,
      )
    };

    Ok(result)
  }

  // returns processed psbt given psbt offer and full unsigned transaction
  fn process_psbt(
    &self,
    wallet: &Wallet,
    psbt: &Psbt,
    unsigned_tx: &Transaction,
    sign: bool,
  ) -> Result<(Psbt, String)> {
    let mut unsigned_psbt = Psbt::from_unsigned_tx(unsigned_tx.clone())?;
    unsigned_psbt
      .inputs
      .splice(1..1 + psbt.inputs.len(), psbt.inputs.clone());
    unsigned_psbt
      .outputs
      .splice(1..1 + psbt.outputs.len(), psbt.outputs.clone());

    let result = wallet.bitcoin_client().call::<String>(
      "utxoupdatepsbt",
      &[base64_encode(&unsigned_psbt.serialize()).into()],
    )?;

    let result = wallet
      .bitcoin_client()
      .wallet_process_psbt(&result, Some(sign), None, None)?;

    ensure! {
      result.complete || !sign,
      "At least 1 PSBT input is unsigned and cannot be signed by wallet"
    }

    let psbt = base64_decode(&result.psbt).context("failed to base64 decode PSBT")?;

    let psbt = Psbt::deserialize(&psbt).context("failed to deserialize PSBT")?;

    Ok((psbt, result.psbt))
  }
}
