use {super::*, crate::wallet::Wallet, std::collections::BTreeSet};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub cardinal: u64,
  pub ordinal: u64,
  pub runes: BTreeMap<Rune, u128>,
  pub runic: u64,
  pub total: u64,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  index.update()?;

  let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;

  let inscription_outputs = index
    .get_inscriptions(&unspent_outputs)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut runes = BTreeMap::new();

  for (outpoint, _amount) in unspent_outputs {
    for (rune, pile) in index.get_rune_balances_for_outpoint(outpoint)? {
      *runes.entry(rune).or_default() += pile.amount;
    }
  }

  let mut cardinal = 0;
  let mut ordinal = 0;
  let mut runic = 0;
  for (outpoint, amount) in index.get_unspent_outputs(Wallet::load(&options)?)? {
    if inscription_outputs.contains(&outpoint) {
      ordinal += amount.to_sat();
    } else if !index.get_rune_balances_for_outpoint(outpoint)?.is_empty() {
      runic += amount.to_sat();
    } else {
      cardinal += amount.to_sat();
    }
  }

  Ok(Box::new(Output {
    cardinal,
    ordinal,
    runes,
    runic,
    total: cardinal + ordinal + runic,
  }))
}
