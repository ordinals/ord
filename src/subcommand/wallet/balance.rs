use {super::*, crate::wallet::Wallet, std::collections::BTreeSet};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub cardinal: u64,
  pub ordinal: u64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub runes: Option<BTreeMap<Rune, u128>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub runic: Option<u64>,
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

  let mut cardinal = 0;
  let mut ordinal = 0;
  let mut runes = BTreeMap::new();
  let mut runic = 0;
  for (outpoint, amount) in unspent_outputs {
    let rune_balances = index.get_rune_balances_for_outpoint(outpoint)?;

    if inscription_outputs.contains(&outpoint) {
      ordinal += amount.to_sat();
    } else if !rune_balances.is_empty() {
      for (rune, pile) in rune_balances {
        *runes.entry(rune).or_default() += pile.amount;
      }
      runic += amount.to_sat();
    } else {
      cardinal += amount.to_sat();
    }
  }

  Ok(Box::new(Output {
    cardinal,
    ordinal,
    runes: index.has_rune_index().then_some(runes),
    runic: index.has_rune_index().then_some(runic),
    total: cardinal + ordinal + runic,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn runes_and_runic_fields_are_not_present_if_none() {
    assert_eq!(
      serde_json::to_string(&Output {
        cardinal: 0,
        ordinal: 0,
        runes: None,
        runic: None,
        total: 0
      })
      .unwrap(),
      r#"{"cardinal":0,"ordinal":0,"total":0}"#
    );
  }
}
