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
  address: Option<Address<NetworkUnchecked>>,
  amount: Amount,
  runes: BTreeMap<SpacedRune, Decimal>,
}

struct Splitfile {
  outputs: Vec<SplitOutput>,
}

struct SplitOutput {
  address: Option<Address>,
  amount: Amount,
  runes: BTreeMap<Rune, u128>,
}

impl Splitfile {
  pub(crate) fn load(
    path: &Path,
    wallet: &Wallet,
  ) -> Result<(Self, BTreeMap<Rune, (RuneId, RuneEntry)>)> {
    let network = wallet.chain().network();

    let unchecked: SplitfileUnchecked = serde_yaml::from_reader(fs::File::open(path)?)?;

    let mut entries = BTreeMap::<Rune, (RuneId, RuneEntry)>::new();

    let mut outputs = Vec::new();

    for output in unchecked.outputs {
      let mut runes = BTreeMap::new();

      for (spaced_rune, decimal) in output.runes {
        let (_id, entry) = if let Some(entry) = entries.get(&spaced_rune.rune) {
          entry
        } else {
          let (id, entry, _parent) = wallet
            .get_rune(spaced_rune.rune)?
            .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;
          entries.insert(spaced_rune.rune, (id, entry));
          entries.get(&spaced_rune.rune).unwrap()
        };

        let amount = decimal.to_integer(entry.divisibility)?;

        assert!(amount != 0);

        runes.insert(spaced_rune.rune, amount);
      }

      outputs.push(SplitOutput {
        address: output
          .address
          .map(|address| address.require_network(network))
          .transpose()?,
        amount: output.amount,
        runes,
      });
    }

    Ok((Self { outputs }, entries))
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
pub struct Output {}

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
          id: ids.get(&rune).unwrap().0,
          amount: *amount,
          output: (i + base).try_into().unwrap(),
        });
      }
    }

    let runestone = Runestone {
      edicts,
      ..default()
    };

    // let unfunded_transaction = Transaction {
    //   version: Version(2),
    //   lock_time: LockTime::ZERO,
    //   input: inputs
    //     .into_iter()
    //     .map(|previous_output| TxIn {
    //       previous_output,
    //       script_sig: ScriptBuf::new(),
    //       sequence: Sequence::MAX,
    //       witness: Witness::new(),
    //     })
    //     .collect(),
    //   output: if needs_runes_change_output {
    //     vec![
    //       TxOut {
    //         script_pubkey: runestone.encipher(),
    //         value: Amount::from_sat(0),
    //       },
    //       TxOut {
    //         script_pubkey: wallet.get_change_address()?.script_pubkey(),
    //         value: postage,
    //       },
    //       TxOut {
    //         script_pubkey: destination.script_pubkey(),
    //         value: postage,
    //       },
    //     ]
    //   } else {
    //     vec![TxOut {
    //       script_pubkey: destination.script_pubkey(),
    //       value: postage,
    //     }]
    //   },
    // };

    // todo:
    // - figure out if we need a change output

    todo!()
  }
}
