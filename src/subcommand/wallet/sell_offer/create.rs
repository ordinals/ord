use {
  super::*, bitcoin::sighash::EcdsaSighashType::SinglePlusAnyoneCanPay,
  bitcoincore_rpc::json::SigHashType,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub outgoing: Vec<Outgoing>,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(long, help = "<INSCRIPTION> or <DECIMAL:RUNE> to make offer for.")]
  outgoing: Vec<Outgoing>,
  #[arg(long, help = "<AMOUNT> to offer.")]
  amount: Amount,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    ensure! {
      self.outgoing.len() == 1,
      "multiple outgoings not yet supported"
    }

    let psbt = match self.outgoing[0] {
      Outgoing::Rune { decimal, rune } => self.create_rune_sell_offer(wallet, decimal, rune)?,
      Outgoing::InscriptionId(_) => bail!("inscription sell offers not yet implemented"),
      _ => bail!("outgoing must be either <INSCRIPTION> or <DECIMAL:RUNE>"),
    };

    Ok(Some(Box::new(Output {
      psbt,
      outgoing: self.outgoing.clone(),
    })))
  }

  fn create_rune_sell_offer(
    &self,
    wallet: Wallet,
    decimal: Decimal,
    spaced_rune: SpacedRune,
  ) -> Result<String> {
    ensure!(
      wallet.has_rune_index(),
      "creating runes offer with `ord offer` requires index created with `--index-runes` flag",
    );

    wallet.lock_non_cardinal_outputs()?;

    let (_id, entry, _parent) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let amount = decimal.to_integer(entry.divisibility)?;

    let inscribed_outputs = wallet
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let balances = wallet
      .get_runic_outputs()?
      .unwrap_or_default()
      .into_iter()
      .filter(|output| !inscribed_outputs.contains(output))
      .map(|output| {
        wallet.get_runes_balances_in_output(&output).map(|balance| {
          (
            output,
            balance
              .unwrap_or_default()
              .into_iter()
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile.amount))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, u128>>>>()?;

    let mut rune_balances = Vec::<u128>::new();
    let mut balance_to_outpoints = BTreeMap::<u128, Vec<OutPoint>>::new();

    for (output, runes) in balances {
      if let Some(balance) = runes.get(&spaced_rune.rune) {
        if runes.len() == 1 {
          rune_balances.push(*balance);
          balance_to_outpoints
            .entry(*balance)
            .or_insert_with(Vec::new)
            .push(output);
        }
      }
    }

    let Some(smallest_subset) = smallest_subset_sum(rune_balances, amount) else {
      bail!(
        "missing outpoint in wallet with exact `{}:{}` balance or set of outpoints summing to `{}:{}`",
        decimal,
        spaced_rune,
        decimal,
        spaced_rune
      );
    };

    let mut inputs = Vec::<OutPoint>::new();
    for balance in &smallest_subset {
      if let Some(outpoint) = balance_to_outpoints.get_mut(balance).unwrap().pop() {
        inputs.push(outpoint);
      }
    }

    let amount_per_output = self.amount.to_sat() / inputs.len() as u64;
    let remainder = usize::try_from(self.amount.to_sat() % inputs.len() as u64).unwrap();

    let mut outputs = Vec::new();
    for i in 0..inputs.len() {
      let postage = wallet.get_value_in_output(&inputs[i])?;

      outputs.push(TxOut {
        value: if i < remainder {
          Amount::from_sat(amount_per_output + postage + 1)
        } else {
          Amount::from_sat(amount_per_output + postage)
        },
        script_pubkey: wallet.get_change_address()?.into(),
      });
    }

    let tx = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: inputs
        .into_iter()
        .map(|previous_output| TxIn {
          previous_output,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        })
        .collect(),
      output: outputs,
    };

    let psbt = Psbt::from_unsigned_tx(tx)?;

    let result = wallet
      .bitcoin_client()
      .call::<String>("utxoupdatepsbt", &[base64_encode(&psbt.serialize()).into()])?;

    let result = wallet.bitcoin_client().wallet_process_psbt(
      &result,
      Some(true),
      Some(SigHashType::from(SinglePlusAnyoneCanPay)),
      None,
    )?;

    ensure! {
      result.complete,
      "Failed to sign PSBT after processing with wallet",
    }

    Ok(result.psbt)
  }
}

pub fn smallest_subset_sum(nums: Vec<u128>, target: u128) -> Option<Vec<u128>> {
  // create a DP table where dp[sum] stores the best subset to reach that sum
  let mut dp: BTreeMap<u128, Vec<u128>> = BTreeMap::new();
  dp.insert(0, Vec::new()); // empty set for sum 0

  // fill the DP table
  for &num in &nums {
    if num <= target {
      let sums: Vec<(u128, Vec<u128>)> = dp
        .iter()
        .filter(|&(k, _)| k + num <= target)
        .map(|(k, v)| (*k, v.clone()))
        .collect();

      // process each existing sum
      for (current_sum, prev_subset) in sums {
        let next_sum = current_sum + num;
        let mut new_subset = prev_subset;
        new_subset.push(num);

        // add subset if new or replace if smaller
        dp.entry(next_sum)
          .and_modify(|existing| {
            if existing.len() > new_subset.len() {
              *existing = new_subset.clone();
            }
          })
          .or_insert(new_subset);
      }
    }
  }

  // return the subset for the target sum if it exists
  dp.get(&target).cloned()
}
