use super::*;

#[derive(Serialize, Deserialize)]
pub struct RunicUtxo {
  pub output: OutPoint,
  pub rune: SpacedRune,
  pub rune_balance: u128,
  pub amount: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();
  let runic_utxos = wallet.get_runic_outputs()?;

  let runic_utxos = unspent_outputs
    .iter()
    .flat_map(|(output, txout)| {
      if runic_utxos.contains(output) {
        wallet
          .get_runes_balances_for_output(output)
          .unwrap()
          .into_iter()
          .map(|(rune, pile)| RunicUtxo {
            rune,
            rune_balance: pile.amount,
            amount: txout.value,
            output: *output,
          })
          .collect()
      } else {
        vec![]
      }
    })
    .collect::<Vec<RunicUtxo>>();

  Ok(Some(Box::new(runic_utxos)))
}