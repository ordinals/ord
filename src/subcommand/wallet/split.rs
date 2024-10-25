use {super::*, splits::Splits};

// todo:
// - test that duplicate keys are an error
// - select runes first, may contain for sats
// - check for dust outputs
// - separate checks and flags for:
//   - runestone over 80 bytes
//   - transaction over 400kwu
// - how to make more efficient?
//   - can omit runes, and add a `--all RUNE AMOUNT` argument, which will use a split
//   - or do this if every output is a getting the same amount
//
// - integration tests:
//   - requires rune index
//   - inputs with inscriptions are not selected

mod splits;

#[derive(Debug, Parser)]
pub(crate) struct Split {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]",
    value_name = "AMOUNT"
  )]
  pub(crate) postage: Option<Amount>,
  #[arg(
    long,
    help = "Split outputs multiple inscriptions and rune defined in YAML <SPLIT_FILE>.",
    value_name = "SPLIT_FILE"
  )]
  pub(crate) splits: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
  pub psbt: String,
  pub fee: u64,
}

impl Split {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet split` requires index created with `--index-runes` flag",
    );

    wallet.lock_non_cardinal_outputs()?;

    let splits = Splits::load(&self.splits, &wallet)?;

    let inscribed_outputs = wallet
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let balances = wallet
      .get_runic_outputs()?
      .into_iter()
      .filter(|output| !inscribed_outputs.contains(output))
      .map(|output| {
        wallet.get_runes_balances_in_output(&output).map(|balance| {
          (
            output,
            balance
              .into_iter()
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile.amount))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, u128>>>>()?;

    let unfunded_transaction =
      Self::build_transaction(balances, wallet.get_change_address()?, &splits);

    let unsigned_transaction = fund_raw_transaction(
      wallet.bitcoin_client(),
      self.fee_rate,
      &unfunded_transaction,
    )?;

    let unsigned_transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    let (txid, psbt, fee) =
      wallet.sign_and_broadcast_transaction(unsigned_transaction, self.dry_run)?;

    Ok(Some(Box::new(Output { txid, psbt, fee })))
  }

  fn build_transaction(
    balances: BTreeMap<OutPoint, BTreeMap<Rune, u128>>,
    change_address: Address,
    splits: &Splits,
  ) -> Transaction {
    let mut input_runes_required = BTreeMap::<Rune, u128>::new();

    for output in &splits.outputs {
      for (rune, amount) in &output.runes {
        let required = input_runes_required.entry(*rune).or_default();
        *required = (*required).checked_add(*amount).unwrap();
      }
    }

    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    let mut inputs = Vec::new();

    for (output, runes) in balances {
      for (rune, required) in &input_runes_required {
        if let Some(balance) = runes.get(rune) {
          assert!(*balance > 0);
          for (rune, balance) in &runes {
            *input_rune_balances.entry(*rune).or_default() += balance;
          }
          inputs.push(output);
          if input_rune_balances.get(rune).cloned().unwrap_or_default() >= *required {
            break;
          }
        }
      }
    }

    let mut need_rune_change_output = false;

    for (rune, balance) in input_rune_balances {
      let required = input_runes_required.get(&rune).copied().unwrap_or_default();
      match balance.cmp(&required) {
        Ordering::Less => {
          todo!("shortfall!");
        }
        Ordering::Greater => {
          need_rune_change_output = true;
        }
        Ordering::Equal => {}
      }
    }

    let mut edicts = Vec::new();

    let base = if need_rune_change_output { 2 } else { 1 };

    for (i, output) in splits.outputs.iter().enumerate() {
      for (rune, amount) in &output.runes {
        edicts.push(Edict {
          id: *splits.rune_ids.get(rune).unwrap(),
          amount: *amount,
          output: (i + base).try_into().unwrap(),
        });
      }
    }

    let runestone = Runestone {
      edicts,
      ..default()
    };

    let mut output = Vec::new();

    let runestone_script_pubkey = runestone.encipher();

    if runestone_script_pubkey.len() > 83 {
      todo!();
    }

    output.push(TxOut {
      script_pubkey: runestone_script_pubkey,
      value: Amount::from_sat(0),
    });

    let postage = Amount::from_sat(10000);

    if need_rune_change_output {
      output.push(TxOut {
        script_pubkey: change_address.script_pubkey(),
        value: postage,
      });
    }

    for split_output in &splits.outputs {
      let script_pubkey = split_output.address.script_pubkey();

      if split_output.value < script_pubkey.minimal_non_dust() {
        todo!();
      }

      output.push(TxOut {
        script_pubkey,
        value: split_output.value,
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
          sequence: Sequence::MAX,
          witness: Witness::new(),
        })
        .collect(),
      output,
    };

    for output in &tx.output {
      if output.value < output.script_pubkey.minimal_non_dust() {
        todo!();
      }
    }

    assert_eq!(
      Runestone::decipher(&tx),
      Some(Artifact::Runestone(runestone)),
    );

    tx
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn foo() {}
}
