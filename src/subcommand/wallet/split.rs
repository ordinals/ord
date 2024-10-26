use {super::*, splits::Splits};

// todo:
// - add an example splits.yaml
//
// - unit tests:
//   - dust outputs are error
//   - postage is used for change output
//   - target_postage is used if postage is omitted
//   - credits multiple runes when output containing multiple runes is selected
//   - doesn't select more outputs than needed
//   - doesn't select fewer outputs than needed
//   - creates change output when non target runes are in selected inputs
//   - creates change output when target runes are in selected inputs
//   - edicts are correct
//   - shoftfall error
//
// - integration tests:
//   - requires rune index
//   - inputs with inscriptions are not selected
//   - no outputs is an error
//   - duplicate keys is an error
//   - tx over 400kwu is an error
//   - mining transaction yields correct result

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

    let unfunded_transaction = Self::build_transaction(
      balances,
      wallet.get_change_address()?,
      self.postage.unwrap_or(TARGET_POSTAGE),
      &splits,
    );

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
    postage: Amount,
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

    if need_rune_change_output {
      output.push(TxOut {
        script_pubkey: change_address.script_pubkey(),
        value: postage,
      });
    }

    for split_output in &splits.outputs {
      let script_pubkey = split_output.address.script_pubkey();
      let minimal_non_dust = script_pubkey.minimal_non_dust();
      let value = split_output.value.unwrap_or(minimal_non_dust);
      output.push(TxOut {
        script_pubkey,
        value,
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
  fn foo() {
    let address = address();
    let change_address = change(0);
    let output = outpoint(0);
    let rune = Rune(0);
    let rune_id = RuneId { block: 1, tx: 1 };

    let balances = [(output, [(rune, 1)].into())].into();

    let splits = Splits {
      outputs: vec![splits::Output {
        address: address.clone(),
        runes: [(rune, 1)].into(),
        value: Some(Amount::from_sat(1000)),
      }],
      rune_ids: [(rune, rune_id)].into(),
    };

    let tx = Split::build_transaction(balances, change_address, Amount::from_sat(1), &splits);

    pretty_assert_eq!(
      tx,
      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
          previous_output: output,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        }],
        output: vec![
          TxOut {
            value: Amount::from_sat(0),
            script_pubkey: Runestone {
              edicts: vec![Edict {
                id: rune_id,
                amount: 1,
                output: 1
              }],
              etching: None,
              mint: None,
              pointer: None,
            }
            .encipher()
          },
          TxOut {
            script_pubkey: address.into(),
            value: Amount::from_sat(1000),
          }
        ],
      },
    );
  }
}
