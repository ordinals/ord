use {super::*, splits::RuneInfo, splits::Splits};

// todo:
// - add an example splits.yaml
// - indicate if a change output was added
// - make this command unstable?
//
// - integration tests:
//   - requires rune index
//   - inputs with inscriptions are not selected
//   - un etched runes is an error
//   - no outputs is an error
//   - duplicate keys is an error
//   - tx over 400kwu is an error
//   - mining transaction yields correct result

#[derive(Debug, PartialEq)]
enum Error {
  NoOutputs,
  Dust {
    value: Amount,
    threshold: Amount,
    output: usize,
  },
  Shortfall {
    rune: Rune,
    have: Pile,
    need: Pile,
  },
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::NoOutputs => write!(f, "split file must contain at least one output"),
      Self::Dust {
        value,
        threshold,
        output,
      } => write!(
        f,
        "output {output} value {value} below dust threshold {threshold}"
      ),
      Self::Shortfall { rune, have, need } => {
        write!(f, "wallet contains {have} of {rune} but need {need}")
      }
    }
  }
}

impl std::error::Error for Error {}

mod splits;

#[derive(Debug, Parser)]
pub(crate) struct Split {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Include <AMOUNT> postage with change output. [default: 10000 sat]",
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
      &wallet.get_change_address()?,
      self.postage,
      &splits,
    )?;

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
    change_address: &Address,
    postage: Option<Amount>,
    splits: &Splits,
  ) -> Result<Transaction, Error> {
    if splits.outputs.is_empty() {
      return Err(Error::NoOutputs);
    }

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
        if input_rune_balances.get(rune).copied().unwrap_or_default() >= *required {
          continue;
        }

        let Some(balance) = runes.get(rune) else {
          continue;
        };

        assert!(*balance > 0);

        for (rune, balance) in &runes {
          *input_rune_balances.entry(*rune).or_default() += balance;
        }

        inputs.push(output);

        break;
      }
    }

    for (&rune, &need) in &input_runes_required {
      let have = input_rune_balances.get(&rune).copied().unwrap_or_default();
      if have < need {
        let info = splits.rune_info[&rune];
        return Err(Error::Shortfall {
          rune,
          have: Pile {
            amount: have,
            divisibility: info.divisibility,
            symbol: info.symbol,
          },
          need: Pile {
            amount: need,
            divisibility: info.divisibility,
            symbol: info.symbol,
          },
        });
      }
    }

    let mut need_rune_change_output = false;
    for (rune, input) in input_rune_balances {
      if input > input_runes_required.get(&rune).copied().unwrap_or_default() {
        need_rune_change_output = true;
      }
    }

    let mut edicts = Vec::new();

    let base = if need_rune_change_output { 2 } else { 1 };

    for (i, output) in splits.outputs.iter().enumerate() {
      for (rune, amount) in &output.runes {
        edicts.push(Edict {
          id: splits.rune_info.get(rune).unwrap().id,
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
        value: postage.unwrap_or(TARGET_POSTAGE),
      });
    }

    for (i, split_output) in splits.outputs.iter().enumerate() {
      let script_pubkey = split_output.address.script_pubkey();
      let threshold = script_pubkey.minimal_non_dust();
      let value = split_output.value.unwrap_or(threshold);
      if value < threshold {
        return Err(Error::Dust {
          output: i,
          threshold,
          value,
        });
      }
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
      assert!(output.value >= output.script_pubkey.minimal_non_dust());
    }

    assert_eq!(
      Runestone::decipher(&tx),
      Some(Artifact::Runestone(runestone)),
    );

    Ok(tx)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // todo:
  // - postage is used for change output
  // - decimals use correct divisibility
  // - target_postage is used if postage is omitted
  // - credits multiple runes when output containing multiple runes is selected
  // - doesn't select more outputs than needed
  // - doesn't select fewer outputs than needed
  // - edicts are correct
  // - shoftfall error

  #[test]
  fn splits_must_have_at_least_one_output() {
    assert_eq!(
      Split::build_transaction(
        BTreeMap::new(),
        &change(0),
        None,
        &Splits {
          outputs: Vec::new(),
          rune_info: BTreeMap::new(),
        },
      )
      .unwrap_err(),
      Error::NoOutputs,
    );
  }

  #[test]
  fn wallet_must_have_enough_runes() {
    assert_eq!(
      Split::build_transaction(
        BTreeMap::new(),
        &change(0),
        None,
        &Splits {
          outputs: vec![splits::Output {
            address: address(),
            runes: [(Rune(0), 1000)].into(),
            value: Some(Amount::from_sat(1000)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 10,
              symbol: Some('@'),
            }
          )]
          .into(),
        },
      )
      .unwrap_err(),
      Error::Shortfall {
        rune: Rune(0),
        have: Pile {
          amount: 0,
          divisibility: 10,
          symbol: Some('@'),
        },
        need: Pile {
          amount: 1000,
          divisibility: 10,
          symbol: Some('@'),
        },
      },
    );

    assert_eq!(
      Split::build_transaction(
        [(outpoint(0), [(Rune(0), 1000)].into())].into(),
        &change(0),
        None,
        &Splits {
          outputs: vec![splits::Output {
            address: address(),
            runes: [(Rune(0), 2000)].into(),
            value: Some(Amount::from_sat(1000)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 2,
              symbol: Some('x'),
            }
          )]
          .into()
        },
      )
      .unwrap_err(),
      Error::Shortfall {
        rune: Rune(0),
        have: Pile {
          amount: 1000,
          divisibility: 2,
          symbol: Some('x'),
        },
        need: Pile {
          amount: 2000,
          divisibility: 2,
          symbol: Some('x'),
        },
      },
    );
  }

  #[test]
  fn split_output_values_may_not_be_dust() {
    assert_eq!(
      Split::build_transaction(
        [(outpoint(0), [(Rune(0), 1000)].into())].into(),
        &change(0),
        None,
        &Splits {
          outputs: vec![splits::Output {
            address: address(),
            runes: [(Rune(0), 1000)].into(),
            value: Some(Amount::from_sat(1)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 0,
              symbol: None,
            }
          )]
          .into(),
        },
      )
      .unwrap_err(),
      Error::Dust {
        value: Amount::from_sat(1),
        threshold: Amount::from_sat(294),
        output: 0,
      }
    );

    assert_eq!(
      Split::build_transaction(
        [(outpoint(0), [(Rune(0), 2000)].into())].into(),
        &change(0),
        None,
        &Splits {
          outputs: vec![
            splits::Output {
              address: address(),
              runes: [(Rune(0), 1000)].into(),
              value: Some(Amount::from_sat(1000)),
            },
            splits::Output {
              address: address(),
              runes: [(Rune(0), 1000)].into(),
              value: Some(Amount::from_sat(10)),
            },
          ],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 0,
              symbol: None,
            }
          )]
          .into()
        },
      )
      .unwrap_err(),
      Error::Dust {
        value: Amount::from_sat(10),
        threshold: Amount::from_sat(294),
        output: 1,
      }
    );
  }

  #[test]
  fn one_output_no_change() {
    let address = address();
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };

    let balances = [(output, [(rune, 1000)].into())].into();

    let splits = Splits {
      outputs: vec![splits::Output {
        address: address.clone(),
        runes: [(rune, 1000)].into(),
        value: None,
      }],
      rune_info: [(
        rune,
        RuneInfo {
          id,
          divisibility: 0,
          symbol: None,
        },
      )]
      .into(),
    };

    let tx = Split::build_transaction(balances, &change(0), None, &splits).unwrap();

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
                id,
                amount: 1000,
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
            value: Amount::from_sat(294),
          }
        ],
      },
    );
  }

  #[test]
  fn one_output_with_change_for_outgoing_rune() {
    let address = address();
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };
    let change = change(0);

    let balances = [(output, [(rune, 2000)].into())].into();

    let splits = Splits {
      outputs: vec![splits::Output {
        address: address.clone(),
        runes: [(rune, 1000)].into(),
        value: None,
      }],
      rune_info: [(
        rune,
        RuneInfo {
          id,
          divisibility: 0,
          symbol: None,
        },
      )]
      .into(),
    };

    let tx = Split::build_transaction(balances, &change, None, &splits).unwrap();

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
                id,
                amount: 1000,
                output: 2
              }],
              etching: None,
              mint: None,
              pointer: None,
            }
            .encipher()
          },
          TxOut {
            script_pubkey: change.into(),
            value: TARGET_POSTAGE,
          },
          TxOut {
            script_pubkey: address.into(),
            value: Amount::from_sat(294),
          }
        ],
      },
    );
  }

  #[test]
  fn one_output_with_change_for_non_outgoing_rune() {
    let address = address();
    let output = outpoint(0);
    let change = change(0);

    let balances = [(output, [(Rune(0), 1000), (Rune(1), 1000)].into())].into();

    let splits = Splits {
      outputs: vec![splits::Output {
        address: address.clone(),
        runes: [(Rune(0), 1000)].into(),
        value: None,
      }],
      rune_info: [(
        Rune(0),
        RuneInfo {
          id: rune_id(0),
          divisibility: 0,
          symbol: None,
        },
      )]
      .into(),
    };

    let tx = Split::build_transaction(balances, &change, None, &splits).unwrap();

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
                id: rune_id(0),
                amount: 1000,
                output: 2
              }],
              etching: None,
              mint: None,
              pointer: None,
            }
            .encipher()
          },
          TxOut {
            script_pubkey: change.into(),
            value: TARGET_POSTAGE,
          },
          TxOut {
            script_pubkey: address.into(),
            value: Amount::from_sat(294),
          }
        ],
      },
    );
  }

  #[test]
  fn outputs_without_value_use_correct_dust_amount() {
    let address = "bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3297"
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .assume_checked();
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };

    let balances = [(output, [(rune, 1000)].into())].into();

    let splits = Splits {
      outputs: vec![splits::Output {
        address: address.clone(),
        runes: [(rune, 1000)].into(),
        value: None,
      }],
      rune_info: [(
        rune,
        RuneInfo {
          id,
          divisibility: 0,
          symbol: None,
        },
      )]
      .into(),
    };

    let tx = Split::build_transaction(balances, &change(0), None, &splits).unwrap();

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
                id,
                amount: 1000,
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
            value: Amount::from_sat(330),
          }
        ],
      },
    );
  }
}
