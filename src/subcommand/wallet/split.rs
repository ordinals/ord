use super::*;

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

#[derive(Deserialize)]
struct SplitfileUnchecked {
  outputs: Vec<SplitOutputUnchecked>,
}

#[derive(Deserialize)]
struct SplitOutputUnchecked {
  address: Address<NetworkUnchecked>,
  value: Amount,
  runes: BTreeMap<SpacedRune, Decimal>,
}

struct Splitfile {
  outputs: Vec<SplitOutput>,
}

struct SplitOutput {
  address: Address,
  value: Amount,
  runes: BTreeMap<Rune, u128>,
}

impl Splitfile {
  pub(crate) fn load(path: &Path, wallet: &Wallet) -> Result<(Self, BTreeMap<Rune, RuneId>)> {
    let network = wallet.chain().network();

    let unchecked: SplitfileUnchecked = serde_yaml::from_reader(File::open(path)?)?;

    let mut entries = BTreeMap::<Rune, (RuneEntry, RuneId)>::new();

    let mut outputs = Vec::new();

    for output in unchecked.outputs {
      let mut runes = BTreeMap::new();

      for (spaced_rune, decimal) in output.runes {
        let (entry, _id) = if let Some(entry) = entries.get(&spaced_rune.rune) {
          entry
        } else {
          let (id, entry, _parent) = wallet
            .get_rune(spaced_rune.rune)?
            .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;
          entries.insert(spaced_rune.rune, (entry, id));
          entries.get(&spaced_rune.rune).unwrap()
        };

        let amount = decimal.to_integer(entry.divisibility)?;

        assert!(amount != 0);

        runes.insert(spaced_rune.rune, amount);
      }

      outputs.push(SplitOutput {
        address: output.address.require_network(network)?,
        value: output.value,
        runes,
      });
    }

    Ok((
      Self { outputs },
      entries
        .into_iter()
        .map(|(rune, (_entry, id))| (rune, id))
        .collect(),
    ))
  }
}

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

    let (splitfile, ids) = Splitfile::load(&self.splits, &wallet)?;

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
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, Pile>>>>()?;

    let mut input_runes_required = BTreeMap::<Rune, u128>::new();

    for output in &splitfile.outputs {
      for (rune, amount) in &output.runes {
        let required = input_runes_required.entry(*rune).or_default();
        *required = (*required).checked_add(*amount).unwrap();
      }
    }

    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    let mut inputs = Vec::new();

    for (output, runes) in balances {
      for (rune, required) in &input_runes_required {
        if let Some(balance) = runes.get(&rune) {
          if balance.amount > 0 {
            for (rune, balance) in &runes {
              *input_rune_balances.entry(*rune).or_default() += balance.amount;
            }
            inputs.push(output);
            if input_rune_balances.get(&rune).cloned().unwrap_or_default() >= *required {
              break;
            }
          }
        }
      }
    }

    let mut need_rune_change_output = false;

    for (rune, required) in &input_runes_required {
      let balance = input_rune_balances.get(rune).copied().unwrap_or_default();

      if balance < *required {
        todo!("shortfall!");
      } else if balance > *required {
        need_rune_change_output = true;
      }
    }

    let mut edicts = Vec::new();

    let base = if need_rune_change_output { 2 } else { 1 };

    for (i, output) in splitfile.outputs.iter().enumerate() {
      for (rune, amount) in &output.runes {
        edicts.push(Edict {
          id: *ids.get(&rune).unwrap(),
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

    output.push(TxOut {
      script_pubkey: runestone.encipher(),
      value: Amount::from_sat(0),
    });

    let postage = Amount::from_sat(10000);

    if need_rune_change_output {
      output.push(TxOut {
        script_pubkey: wallet.get_change_address()?.script_pubkey(),
        value: postage,
      });
    }

    for split_output in splitfile.outputs {
      output.push(TxOut {
        script_pubkey: split_output.address.into(),
        value: split_output.value,
      });
    }

    let unfunded_transaction = Transaction {
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

    let unsigned_transaction = fund_raw_transaction(
      wallet.bitcoin_client(),
      self.fee_rate,
      &unfunded_transaction,
    )?;

    let unsigned_transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    assert_eq!(
      Runestone::decipher(&unsigned_transaction),
      Some(Artifact::Runestone(runestone)),
    );

    let (txid, psbt, fee) = wallet.sign_transaction(unsigned_transaction, self.dry_run)?;

    Ok(Some(Box::new(Output { txid, psbt, fee })))
  }
}
