use {super::*, std::collections::BTreeSet};

#[derive(Serialize, Deserialize)]
pub struct CardinalUtxo {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();

  let inscribed_utxos = wallet
    .inscriptions()
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let cardinal_utxos = unspent_outputs
    .iter()
    .filter_map(|(output, txout)| {
      if inscribed_utxos.contains(output) {
        None
      } else {
        Some(CardinalUtxo {
          output: *output,
          amount: txout.value,
        })
      }
    })
    .collect::<Vec<CardinalUtxo>>();

  Ok(Some(Box::new(cardinal_utxos)))
}
