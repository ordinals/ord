use super::*;

#[derive(Deserialize)]
struct SplitsUnchecked {
  outputs: Vec<OutputUnchecked>,
}

#[derive(Deserialize)]
struct OutputUnchecked {
  address: Address<NetworkUnchecked>,
  value: Amount,
  runes: BTreeMap<SpacedRune, Decimal>,
}

pub(crate) struct Splits {
  pub(crate) outputs: Vec<Output>,
  pub(crate) rune_ids: BTreeMap<Rune, RuneId>,
}

pub(crate) struct Output {
  pub(crate) address: Address,
  pub(crate) value: Amount,
  pub(crate) runes: BTreeMap<Rune, u128>,
}

impl Splits {
  pub(crate) fn load(path: &Path, wallet: &Wallet) -> Result<Self> {
    let network = wallet.chain().network();

    let unchecked: SplitsUnchecked = serde_yaml::from_reader(File::open(path)?)?;

    ensure! {
      !unchecked.outputs.is_empty(),
      "splits must contain at least one output",
    }

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

      outputs.push(Output {
        address: output.address.require_network(network)?,
        value: output.value,
        runes,
      });
    }

    Ok(Self {
      outputs,
      rune_ids: entries
        .into_iter()
        .map(|(rune, (_entry, id))| (rune, id))
        .collect(),
    })
  }
}
