use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub cardinal: u64,
  pub ordinal: u64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub runes: Option<BTreeMap<SpacedRune, Decimal>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub runic: Option<u64>,
  pub total: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();

  let inscription_outputs = wallet
    .inscriptions()
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut cardinal = 0;
  let mut ordinal = 0;
  let mut runes = BTreeMap::new();
  let mut runic = 0;

  for (output, txout) in unspent_outputs {
    let rune_balances = wallet
      .get_runes_balances_in_output(output)?
      .unwrap_or_default();

    let is_ordinal = inscription_outputs.contains(output);
    let is_runic = !rune_balances.is_empty();

    if is_ordinal {
      ordinal += txout.value.to_sat();
    }

    if is_runic {
      for (spaced_rune, pile) in rune_balances {
        runes
          .entry(spaced_rune)
          .and_modify(|decimal: &mut Decimal| {
            assert_eq!(decimal.scale, pile.divisibility);
            decimal.value += pile.amount;
          })
          .or_insert(Decimal {
            value: pile.amount,
            scale: pile.divisibility,
          });
      }
      runic += txout.value.to_sat();
    }

    if !is_ordinal && !is_runic {
      cardinal += txout.value.to_sat();
    }

    if is_ordinal && is_runic {
      eprintln!("warning: output {output} contains both inscriptions and runes");
    }
  }

  Ok(Some(Box::new(Output {
    cardinal,
    ordinal,
    runes: wallet.has_rune_index().then_some(runes),
    runic: wallet.has_rune_index().then_some(runic),
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
