use {super::*, std::collections::BTreeSet};

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

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.get_unspent_outputs()?;

  let inscription_outputs = wallet
    .get_inscriptions()?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut cardinal = 0;
  let mut ordinal = 0;
  let mut runes = BTreeMap::new();
  let mut runic = 0;

  for (output, amount) in unspent_outputs {
    let rune_balances = wallet.get_runes_balances_for_output(&output)?;

    if inscription_outputs.contains(&output) {
      ordinal += amount.to_sat();
    } else if !rune_balances.is_empty() {
      for (spaced_rune, pile) in rune_balances {
        *runes.entry(spaced_rune.rune).or_default() += pile.amount;
      }
      runic += amount.to_sat();
    } else {
      cardinal += amount.to_sat();
    }
  }

  Ok(Some(Box::new(Output {
    cardinal,
    ordinal,
    runes: wallet.get_server_status()?.rune_index.then_some(runes),
    runic: wallet.get_server_status()?.rune_index.then_some(runic),
    total: cardinal + ordinal + runic,
  })))
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
