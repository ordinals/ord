use {
  super::*, bitcoin::sighash::EcdsaSighashType::SinglePlusAnyoneCanPay,
  bitcoincore_rpc::json::SigHashType,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub outgoing: Vec<Outgoing>,
  pub partial: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(long, help = "<INSCRIPTION> or <DECIMAL:RUNE> to make offer for.")]
  outgoing: Vec<Outgoing>,
  #[arg(long, help = "<AMOUNT> to offer.")]
  amount: Amount,
  #[arg(long, help = "Allow partial offers if exact balance does not exist.")]
  allow_partial: bool,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    ensure! {
      self.outgoing.len() == 1,
      "multiple outgoings not yet supported"
    }

    let (psbt, outgoing, partial) = match self.outgoing[0] {
      Outgoing::Rune { decimal, rune } => self.create_rune_sell_offer(wallet, decimal, rune)?,
      Outgoing::InscriptionId(_) => bail!("inscription sell offers not yet implemented"),
      _ => bail!("outgoing must be either <INSCRIPTION> or <DECIMAL:RUNE>"),
    };

    Ok(Some(Box::new(Output {
      psbt,
      outgoing: vec![outgoing],
      partial,
    })))
  }

  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::cast_sign_loss)]
  fn create_rune_sell_offer(
    &self,
    wallet: Wallet,
    decimal: Decimal,
    spaced_rune: SpacedRune,
  ) -> Result<(String, Outgoing, bool)> {
    ensure!(
      wallet.has_rune_index(),
      "creating runes offer with `ord offer` requires index created with `--index-runes` flag",
    );

    wallet.lock_non_cardinal_outputs()?;

    let (_id, entry, _parent) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let rune_amount = decimal.to_integer(entry.divisibility)?;

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
            .or_default()
            .push(output);
        }
      }
    }

    if rune_balances.is_empty() {
      bail!(
        "missing outpoint in wallet with only a `{}` balance",
        spaced_rune
      );
    }

    let (subset, sum) = find_best_knapsack(rune_balances, rune_amount);
    let partial = sum < rune_amount;
    let mut sat_amount = self.amount.to_sat();

    if partial {
      if self.allow_partial {
        // proportionally round down required sats
        sat_amount = ((sat_amount as f64 * sum as f64) / rune_amount as f64).round() as u64;
      } else {
        bail!(
          "missing outpoint in wallet with exact `{}:{}` balance or set of outpoints summing to `{}:{}` (try using --allow-partial)",
          decimal,
          spaced_rune,
          decimal,
          spaced_rune
        );
      }
    }

    let mut inputs = Vec::<OutPoint>::new();
    for balance in &subset {
      if let Some(outpoint) = balance_to_outpoints.get_mut(balance).unwrap().pop() {
        inputs.push(outpoint);
      }
    }

    let amount_per_output = sat_amount / inputs.len() as u64;
    let remainder = usize::try_from(sat_amount % inputs.len() as u64).unwrap();

    let mut outputs = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
      let postage = wallet.get_value_in_output(input)?;

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

    let outgoing = Outgoing::Rune {
      decimal: Decimal {
        value: sum,
        scale: decimal.scale,
      },
      rune: spaced_rune,
    };

    Ok((result.psbt, outgoing, partial))
  }
}

// classic knapsack algorithm, optimized to choose smallest subset that sums to the largest value at or below target
pub fn find_best_knapsack(nums: Vec<u128>, target: u128) -> (Vec<u128>, u128) {
  // create a DP table where dp[sum] stores the smallest subset to reach that sum
  let mut dp: BTreeMap<u128, Vec<u128>> = BTreeMap::new();
  dp.insert(0, Vec::new()); // empty set for sum 0

  let mut max_sum = 0;

  // fill the DP table
  for &num in &nums {
    if num <= target {
      let sums: Vec<(u128, Vec<u128>)> = dp
        .iter()
        .filter(|&(k, _)| k + num <= target)
        .map(|(&k, v)| (k, v.clone()))
        .collect();

      // process each existing sum
      for (sum, subset) in sums {
        let new_sum = sum + num;
        let mut new_subset = subset;
        new_subset.push(num);

        // add subset if new or replace if smaller
        dp.entry(new_sum)
          .and_modify(|existing| {
            if existing.len() > new_subset.len() {
              *existing = new_subset.clone();
            }
          })
          .or_insert(new_subset);

        if new_sum > max_sum {
          max_sum = new_sum;
        }
      }
    }
  }

  (dp.get(&max_sum).unwrap().clone(), max_sum)
}
