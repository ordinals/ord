use super::*;

#[derive(Serialize, Deserialize)]
pub struct RunicUtxo {
  pub output: OutPoint,
  pub runes: BTreeMap<SpacedRune, Decimal>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();
  let runic_utxos = wallet.get_runic_outputs()?;

  let runic_utxos = unspent_outputs
    .iter()
    .filter_map(|(output, _)| {
      if runic_utxos.contains(output) {
        let rune_balances = wallet.get_runes_balances_in_output(output).ok()?;
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

        Some(RunicUtxo {
          output: *output,
          runes,
        })
      } else {
        None
      }
    })
    .collect::<Vec<RunicUtxo>>();

  Ok(Some(Box::new(runic_utxos)))
}
