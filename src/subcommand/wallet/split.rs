use {super::*, splitfile::Splitfile};

mod splitfile;

#[derive(Debug, PartialEq)]
enum Error {
  DustOutput {
    value: Amount,
    threshold: Amount,
    output: usize,
  },
  DustPostage {
    value: Amount,
    threshold: Amount,
  },
  NoOutputs,
  RunestoneSize {
    size: usize,
  },
  Shortfall {
    rune: SpacedRune,
    have: Pile,
    need: Pile,
  },
  TransactionSize {
    weight: u64,
  },
  ZeroValue {
    output: usize,
    rune: SpacedRune,
  },
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::DustOutput {
        value,
        threshold,
        output,
      } => write!(
        f,
        "output {output} value {value} below dust threshold {threshold}"
      ),
      Self::DustPostage { value, threshold } => {
        write!(f, "postage value {value} below dust threshold {threshold}")
      }
      Self::NoOutputs => write!(f, "split file must contain at least one output"),
      Self::RunestoneSize { size } => write!(
        f,
        "runestone size {size} over maximum standard OP_RETURN size {MAX_STANDARD_OP_RETURN_SIZE}"
      ),
      Self::Shortfall { rune, have, need } => {
        write!(f, "wallet contains {have} of {rune} but need {need}")
      }
      Self::TransactionSize { weight } => write!(
        f,
        "transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {weight}"
      ),
      Self::ZeroValue { output, rune } => {
        write!(f, "output {output} has zero value for rune {rune}")
      }
    }
  }
}

impl std::error::Error for Error {}

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
  #[arg(
    long,
    alias = "nolimit",
    help = "Allow OP_RETURN greater than 83 bytes. Transactions over this limit are nonstandard \
    and will not be relayed by bitcoind in its default configuration. Do not use this flag unless \
    you understand the implications."
  )]
  pub(crate) no_limit: bool,
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
      "`ord wallet split` requires index created with `--index-runes`",
    );

    wallet.lock_non_cardinal_outputs()?;

    let splits = Splitfile::load(&self.splits, &wallet)?;

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

    let (unfunded_transaction, needs_change) = Self::build_transaction(
      self.no_limit,
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

    let unsigned_transaction: Transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    assert_eq!(
      unsigned_transaction.output.len(),
      splits.outputs.len() + 2 + usize::from(needs_change)
    );

    let (txid, psbt, fee) =
      wallet.sign_and_broadcast_transaction(unsigned_transaction, self.dry_run)?;

    Ok(Some(Box::new(Output { txid, psbt, fee })))
  }

  fn build_transaction(
    no_limit: bool,
    balances: BTreeMap<OutPoint, BTreeMap<Rune, u128>>,
    change_address: &Address,
    postage: Option<Amount>,
    splits: &Splitfile,
  ) -> Result<(Transaction, bool), Error> {
    if splits.outputs.is_empty() {
      return Err(Error::NoOutputs);
    }

    let postage = postage.unwrap_or(TARGET_POSTAGE);

    let change_script_pubkey = change_address.script_pubkey();

    let change_dust_threshold = change_script_pubkey.minimal_non_dust();

    if postage < change_script_pubkey.minimal_non_dust() {
      return Err(Error::DustPostage {
        value: postage,
        threshold: change_dust_threshold,
      });
    }

    let mut input_runes_required = BTreeMap::<Rune, u128>::new();

    for (i, output) in splits.outputs.iter().enumerate() {
      for (&rune, &amount) in &output.runes {
        if amount == 0 {
          return Err(Error::ZeroValue {
            rune: splits.rune_info[&rune].spaced_rune,
            output: i,
          });
        }
        let required = input_runes_required.entry(rune).or_default();
        *required = (*required).checked_add(amount).unwrap();
      }
    }

    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    let mut inputs = Vec::new();

    for (output, runes) in balances {
      for (rune, required) in &input_runes_required {
        if input_rune_balances.get(rune).copied().unwrap_or_default() >= *required {
          continue;
        }

        if runes.get(rune).copied().unwrap_or_default() == 0 {
          continue;
        }

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
          rune: info.spaced_rune,
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

    let mut rune_change_output = None;
    for (rune, input) in &input_rune_balances {
      if input > &input_runes_required.get(rune).copied().unwrap_or_default() {
        // add OP_RETURN
        rune_change_output = Some(u32::try_from(splits.outputs.len()).unwrap() + 1);
      }
    }

    let mut edicts = Vec::new();

    let output_count = if let Some(change_output) = rune_change_output {
      change_output + 2
    } else {
      u32::try_from(splits.outputs.len()).unwrap() + 2
    };

    for (i, output) in splits.outputs.iter().enumerate() {
      for (rune, amount) in &output.runes {
        edicts.push(Edict {
          id: splits.rune_info.get(rune).unwrap().id,
          amount: *amount,
          output: (i + 1).try_into().unwrap(),
        });
      }
    }

    let mut use_even = false;

    if let Some((rune, amount)) = splits.even() {
      let have = input_rune_balances.get(&rune).copied().unwrap_or_default();
      let need = input_runes_required.get(&rune).copied().unwrap_or_default();

      let change = have - need;

      let mut even_edicts = Vec::new();

      let id = splits.rune_info.get(&rune).unwrap().id;

      if let Some(output) = rune_change_output {
        even_edicts.push(Edict {
          id,
          amount: change,
          output,
        });
      }

      even_edicts.push(Edict {
        id,
        amount,
        output: output_count,
      });

      if even_edicts.len() < edicts.len() {
        edicts = even_edicts;
        use_even = true;
      }
    }

    // should this be inside a builder function for the Runestone?
    edicts.sort_by_key(|edict| edict.id);

    let runestone = Runestone {
      edicts,
      pointer: rune_change_output,
      ..default()
    };

    let mut outputs = Vec::new();

    let runestone_script_pubkey = runestone.encipher();
    let size = runestone_script_pubkey.len();

    if !no_limit && size > MAX_STANDARD_OP_RETURN_SIZE {
      return Err(Error::RunestoneSize { size });
    }

    outputs.push(TxOut {
      script_pubkey: runestone_script_pubkey,
      value: Amount::from_sat(0),
    });

    for (i, split_output) in splits.outputs.iter().enumerate() {
      let script_pubkey = split_output.address.script_pubkey();
      let threshold = script_pubkey.minimal_non_dust();
      let value = split_output.value.unwrap_or(threshold);
      if value < threshold {
        return Err(Error::DustOutput {
          output: i,
          threshold,
          value,
        });
      }
      outputs.push(TxOut {
        script_pubkey,
        value,
      });
    }

    if rune_change_output.is_some() {
      outputs.push(TxOut {
        script_pubkey: change_script_pubkey,
        value: postage,
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
      output: outputs,
    };

    let weight = tx.weight().to_vbytes_ceil();

    if !no_limit && weight > MAX_STANDARD_TX_WEIGHT.into() {
      return Err(Error::TransactionSize { weight });
    }

    for output in &tx.output {
      assert!(output.value >= output.script_pubkey.minimal_non_dust());
    }

    // still missing the fundrawtransactions change output
    assert_eq!(output_count, u32::try_from(tx.output.len()).unwrap() + 1);

    // this should complain because we haven't added the change output from fundrawtransaction
    if use_even {
      assert_eq!(
        Runestone::decipher(&tx),
        Some(Artifact::Cenotaph(ordinals::Cenotaph {
          etching: None,
          flaw: Some(ordinals::Flaw::EdictOutput),
          mint: None
        })),
      );
    } else {
      assert_eq!(
        Runestone::decipher(&tx),
        Some(Artifact::Runestone(runestone)),
      );
    }

    Ok((tx, rune_change_output.is_some()))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, splitfile::RuneInfo};

  #[test]
  fn splits_must_have_at_least_one_output() {
    assert_eq!(
      Split::build_transaction(
        false,
        BTreeMap::new(),
        &change(0),
        None,
        &Splitfile {
          outputs: Vec::new(),
          rune_info: BTreeMap::new(),
        },
      )
      .unwrap_err(),
      Error::NoOutputs,
    );
  }

  #[test]
  fn postage_may_not_be_dust() {
    assert_eq!(
      Split::build_transaction(
        false,
        BTreeMap::new(),
        &change(0),
        Some(Amount::from_sat(100)),
        &Splitfile {
          outputs: vec![splitfile::Output {
            address: address(0),
            runes: [(Rune(0), 1000)].into(),
            value: Some(Amount::from_sat(1000)),
          }],
          rune_info: BTreeMap::new(),
        },
      )
      .unwrap_err(),
      Error::DustPostage {
        value: Amount::from_sat(100),
        threshold: Amount::from_sat(294),
      },
    );
  }

  #[test]
  fn output_rune_value_may_not_be_zero() {
    assert_eq!(
      Split::build_transaction(
        false,
        BTreeMap::new(),
        &change(0),
        None,
        &Splitfile {
          outputs: vec![splitfile::Output {
            address: address(0),
            runes: [(Rune(0), 0)].into(),
            value: Some(Amount::from_sat(1000)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 10,
              symbol: Some('@'),
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 1,
              },
            },
          )]
          .into()
        },
      )
      .unwrap_err(),
      Error::ZeroValue {
        output: 0,
        rune: SpacedRune {
          rune: Rune(0),
          spacers: 1,
        },
      },
    );

    assert_eq!(
      Split::build_transaction(
        false,
        BTreeMap::new(),
        &change(0),
        None,
        &Splitfile {
          outputs: vec![
            splitfile::Output {
              address: address(0),
              runes: [(Rune(0), 100)].into(),
              value: Some(Amount::from_sat(1000)),
            },
            splitfile::Output {
              address: address(0),
              runes: [(Rune(0), 0)].into(),
              value: Some(Amount::from_sat(1000)),
            },
          ],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 10,
              symbol: Some('@'),
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 10,
              },
            },
          )]
          .into()
        },
      )
      .unwrap_err(),
      Error::ZeroValue {
        output: 1,
        rune: SpacedRune {
          rune: Rune(0),
          spacers: 10,
        },
      },
    );
  }

  #[test]
  fn wallet_must_have_enough_runes() {
    assert_eq!(
      Split::build_transaction(
        false,
        BTreeMap::new(),
        &change(0),
        None,
        &Splitfile {
          outputs: vec![splitfile::Output {
            address: address(0),
            runes: [(Rune(0), 1000)].into(),
            value: Some(Amount::from_sat(1000)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 10,
              symbol: Some('@'),
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 2,
              },
            },
          )]
          .into(),
        },
      )
      .unwrap_err(),
      Error::Shortfall {
        rune: SpacedRune {
          rune: Rune(0),
          spacers: 2
        },
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
        false,
        [(outpoint(0), [(Rune(0), 1000)].into())].into(),
        &change(0),
        None,
        &Splitfile {
          outputs: vec![splitfile::Output {
            address: address(0),
            runes: [(Rune(0), 2000)].into(),
            value: Some(Amount::from_sat(1000)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 2,
              symbol: Some('x'),
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 1
              },
            },
          )]
          .into()
        },
      )
      .unwrap_err(),
      Error::Shortfall {
        rune: SpacedRune {
          rune: Rune(0),
          spacers: 1,
        },
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
        false,
        [(outpoint(0), [(Rune(0), 1000)].into())].into(),
        &change(0),
        None,
        &Splitfile {
          outputs: vec![splitfile::Output {
            address: address(0),
            runes: [(Rune(0), 1000)].into(),
            value: Some(Amount::from_sat(1)),
          }],
          rune_info: [(
            Rune(0),
            RuneInfo {
              id: RuneId { block: 1, tx: 1 },
              divisibility: 0,
              symbol: None,
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 0,
              },
            },
          )]
          .into(),
        },
      )
      .unwrap_err(),
      Error::DustOutput {
        value: Amount::from_sat(1),
        threshold: Amount::from_sat(294),
        output: 0,
      }
    );

    assert_eq!(
      Split::build_transaction(
        false,
        [(outpoint(0), [(Rune(0), 2000)].into())].into(),
        &change(0),
        None,
        &Splitfile {
          outputs: vec![
            splitfile::Output {
              address: address(0),
              runes: [(Rune(0), 1000)].into(),
              value: Some(Amount::from_sat(1000)),
            },
            splitfile::Output {
              address: address(0),
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
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 0,
              },
            },
          )]
          .into()
        },
      )
      .unwrap_err(),
      Error::DustOutput {
        value: Amount::from_sat(10),
        threshold: Amount::from_sat(294),
        output: 1,
      }
    );
  }

  #[test]
  fn one_output_no_change() {
    let address = address(0);
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };

    let balances = [(output, [(rune, 1000)].into())].into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
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
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change(0), None, &splits).unwrap();

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
  fn one_output_with_change_for_outgoing_rune_with_default_postage() {
    let address = address(0);
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };
    let change = change(0);

    let balances = [(output, [(rune, 2000)].into())].into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
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
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change, None, &splits).unwrap();

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
              pointer: Some(2),
            }
            .encipher()
          },
          TxOut {
            script_pubkey: address.into(),
            value: Amount::from_sat(294),
          },
          TxOut {
            script_pubkey: change.into(),
            value: TARGET_POSTAGE,
          },
        ],
      },
    );
  }

  #[test]
  fn one_output_with_change_for_outgoing_rune_with_non_default_postage() {
    let address = address(0);
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };
    let change = change(0);

    let balances = [(output, [(rune, 2000)].into())].into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
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
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(
      false,
      balances,
      &change,
      Some(Amount::from_sat(500)),
      &splits,
    )
    .unwrap();

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
              pointer: Some(2),
            }
            .encipher()
          },
          TxOut {
            script_pubkey: address.into(),
            value: Amount::from_sat(294),
          },
          TxOut {
            script_pubkey: change.into(),
            value: Amount::from_sat(500),
          },
        ],
      },
    );
  }

  #[test]
  fn one_output_with_change_for_non_outgoing_rune() {
    let address = address(0);
    let output = outpoint(0);
    let change = change(0);

    let balances = [(output, [(Rune(0), 1000), (Rune(1), 1000)].into())].into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
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
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change, None, &splits).unwrap();

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
                output: 1
              }],
              etching: None,
              mint: None,
              pointer: Some(2),
            }
            .encipher()
          },
          TxOut {
            script_pubkey: address.into(),
            value: Amount::from_sat(294),
          },
          TxOut {
            script_pubkey: change.into(),
            value: TARGET_POSTAGE,
          },
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

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
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
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change(0), None, &splits).unwrap();

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

  #[test]
  fn excessive_inputs_are_not_selected() {
    let address = address(0);
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };

    let balances = [
      (output, [(rune, 1000)].into()),
      (outpoint(1), [(rune, 1000)].into()),
    ]
    .into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
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
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change(0), None, &splits).unwrap();

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
  fn multiple_inputs_may_be_selected() {
    let address = address(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };

    let balances = [
      (outpoint(0), [(rune, 1000)].into()),
      (outpoint(1), [(rune, 1000)].into()),
    ]
    .into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
        address: address.clone(),
        runes: [(rune, 2000)].into(),
        value: None,
      }],
      rune_info: [(
        rune,
        RuneInfo {
          id,
          divisibility: 0,
          symbol: None,
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change(0), None, &splits).unwrap();

    pretty_assert_eq!(
      tx,
      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![
          TxIn {
            previous_output: outpoint(0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          },
          TxIn {
            previous_output: outpoint(1),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          },
        ],
        output: vec![
          TxOut {
            value: Amount::from_sat(0),
            script_pubkey: Runestone {
              edicts: vec![Edict {
                id,
                amount: 2000,
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
  fn two_outputs_no_change() {
    let output = outpoint(0);
    let rune = Rune(0);
    let id = RuneId { block: 1, tx: 1 };

    let balances = [(output, [(rune, 1000)].into())].into();

    let splits = Splitfile {
      outputs: vec![
        splitfile::Output {
          address: address(0),
          runes: [(rune, 800)].into(),
          value: None,
        },
        splitfile::Output {
          address: address(1),
          runes: [(rune, 200)].into(),
          value: None,
        },
      ],
      rune_info: [(
        rune,
        RuneInfo {
          id,
          divisibility: 0,
          symbol: None,
          spaced_rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
        },
      )]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change(0), None, &splits).unwrap();

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
              edicts: vec![
                Edict {
                  id,
                  amount: 800,
                  output: 1
                },
                Edict {
                  id,
                  amount: 200,
                  output: 2
                }
              ],
              etching: None,
              mint: None,
              pointer: None,
            }
            .encipher()
          },
          TxOut {
            script_pubkey: address(0).into(),
            value: Amount::from_sat(294),
          },
          TxOut {
            script_pubkey: address(1).into(),
            value: Amount::from_sat(294),
          }
        ],
      },
    );
  }

  #[test]
  fn outputs_may_receive_multiple_runes() {
    let address = address(0);

    let balances = [
      (outpoint(0), [(Rune(0), 1000)].into()),
      (outpoint(1), [(Rune(1), 2000)].into()),
    ]
    .into();

    let splits = Splitfile {
      outputs: vec![splitfile::Output {
        address: address.clone(),
        runes: [(Rune(0), 1000), (Rune(1), 2000)].into(),
        value: None,
      }],
      rune_info: [
        (
          Rune(0),
          RuneInfo {
            id: rune_id(0),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(0),
              spacers: 0,
            },
          },
        ),
        (
          Rune(1),
          RuneInfo {
            id: rune_id(1),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(1),
              spacers: 0,
            },
          },
        ),
      ]
      .into(),
    };

    let (tx, _) = Split::build_transaction(false, balances, &change(0), None, &splits).unwrap();

    pretty_assert_eq!(
      tx,
      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![
          TxIn {
            previous_output: outpoint(0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          },
          TxIn {
            previous_output: outpoint(1),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          },
        ],
        output: vec![
          TxOut {
            value: Amount::from_sat(0),
            script_pubkey: Runestone {
              edicts: vec![
                Edict {
                  id: rune_id(0),
                  amount: 1000,
                  output: 1
                },
                Edict {
                  id: rune_id(1),
                  amount: 2000,
                  output: 1
                },
              ],
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
  fn oversize_op_return_is_an_error() {
    let balances = [
      (outpoint(0), [(Rune(0), 5_000_000_000)].into()),
      (outpoint(1), [(Rune(1), 5_000_000_000)].into()),
    ]
    .into();

    let splits = Splitfile {
      outputs: (0..10)
        .map(|i| splitfile::Output {
          address: address(i).clone(),
          runes: [(Rune(u128::from(i) % 2), 1_000_000_000)].into(),
          value: None,
        })
        .collect(),
      rune_info: [
        (
          Rune(0),
          RuneInfo {
            id: rune_id(0),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(0),
              spacers: 0,
            },
          },
        ),
        (
          Rune(1),
          RuneInfo {
            id: rune_id(1),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(1),
              spacers: 0,
            },
          },
        ),
      ]
      .into(),
    };

    pretty_assert_eq!(
      Split::build_transaction(false, balances, &change(0), None, &splits).unwrap_err(),
      Error::RunestoneSize { size: 85 },
    );
  }

  #[test]
  fn oversize_op_return_is_allowed_with_flag() {
    let balances = [
      (outpoint(0), [(Rune(0), 5_000_000_000)].into()),
      (outpoint(1), [(Rune(1), 5_000_000_000)].into()),
    ]
    .into();

    let splits = Splitfile {
      outputs: (0..10)
        .map(|i| splitfile::Output {
          address: address(i).clone(),
          runes: [(Rune(u128::from(i) % 2), 1_000_000_000)].into(),
          value: None,
        })
        .collect(),
      rune_info: [
        (
          Rune(0),
          RuneInfo {
            id: rune_id(0),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(0),
              spacers: 0,
            },
          },
        ),
        (
          Rune(1),
          RuneInfo {
            id: rune_id(1),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(1),
              spacers: 0,
            },
          },
        ),
      ]
      .into(),
    };

    let (tx, _) = Split::build_transaction(true, balances, &change(0), None, &splits).unwrap();

    pretty_assert_eq!(
      tx,
      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![
          TxIn {
            previous_output: outpoint(0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          },
          TxIn {
            previous_output: outpoint(1),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          }
        ],
        output: (0..11)
          .map(|i| if i == 0 {
            TxOut {
              value: Amount::from_sat(0),
              script_pubkey: Runestone {
                edicts: (0..10)
                  .map(|i| Edict {
                    id: rune_id((i % 2 != 0).into()),
                    amount: 1_000_000_000,
                    output: i + 1,
                  })
                  .collect(),
                etching: None,
                mint: None,
                pointer: None,
              }
              .encipher(),
            }
          } else {
            TxOut {
              script_pubkey: address(i - 1).into(),
              value: Amount::from_sat(294),
            }
          })
          .collect()
      }
    );
  }

  #[test]
  fn even_split_with_change() {
    let balances = [
      (outpoint(0), [(Rune(0), 5_000_000_000)].into()),
      (outpoint(1), [(Rune(1), 13_000_000_000)].into()),
    ]
    .into();

    let splits = Splitfile {
      outputs: (0..10)
        .map(|i| splitfile::Output {
          address: address(i).clone(),
          runes: [(Rune(1), 1_000_000_000)].into(),
          value: None,
        })
        .collect(),
      rune_info: [
        (
          Rune(0),
          RuneInfo {
            id: rune_id(0),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(0),
              spacers: 0,
            },
          },
        ),
        (
          Rune(1),
          RuneInfo {
            id: rune_id(1),
            divisibility: 0,
            symbol: None,
            spaced_rune: SpacedRune {
              rune: Rune(1),
              spacers: 0,
            },
          },
        ),
      ]
      .into(),
    };

    let (tx, _) = Split::build_transaction(true, balances, &change(0), None, &splits).unwrap();

    pretty_assert_eq!(
      tx,
      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
          previous_output: outpoint(1),
          script_sig: ScriptBuf::new(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        }],
        output: (0..12)
          .map(|i| if i == 0 {
            TxOut {
              value: Amount::from_sat(0),
              script_pubkey: Runestone {
                edicts: vec![
                  Edict {
                    id: rune_id(1),
                    amount: 3_000_000_000,
                    output: 11,
                  },
                  Edict {
                    id: rune_id(1),
                    amount: 1_000_000_000,
                    output: 13,
                  },
                ],
                etching: None,
                mint: None,
                pointer: Some(11),
              }
              .encipher(),
            }
          } else if i == 11 {
            TxOut {
              script_pubkey: change(0).into(),
              value: TARGET_POSTAGE,
            }
          } else {
            TxOut {
              script_pubkey: address(i - 1).into(),
              value: Amount::from_sat(294),
            }
          })
          .collect()
      }
    );
  }
}
