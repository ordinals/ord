use super::*;

#[derive(Serialize, Deserialize)]
pub struct RunicUtxo {
  pub output: OutPoint,
  pub runes: BTreeMap<SpacedRune, Decimal>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();
  let Some(runic_utxos) = wallet.get_runic_outputs()? else {
    bail!("`ord wallet runics` requires index created with `--index-runes`")
  };

  let mut result = Vec::new();

  for output in unspent_outputs.keys() {
    if runic_utxos.contains(output) {
      let rune_balances = wallet
        .get_runes_balances_in_output(output)?
        .unwrap_or_default();

      let mut runes = BTreeMap::new();

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

      result.push(RunicUtxo {
        output: *output,
        runes,
      });
    }
  }

  Ok(Some(Box::new(result)))
}
