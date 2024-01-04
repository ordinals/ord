use {super::*, std::collections::BTreeSet};

#[derive(Serialize, Deserialize)]
pub struct CardinalUtxo {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.get_unspent_outputs()?;

  let inscribed_utxos = wallet
    .get_inscriptions()?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let cardinal_utxos = unspent_outputs
    .iter()
    .filter_map(|(output, amount)| {
      if inscribed_utxos.contains(output) {
        None
      } else {
        Some(CardinalUtxo {
          output: *output,
          amount: amount.to_sat(),
        })
      }
    })
    .collect::<Vec<CardinalUtxo>>();

  Ok(Box::new(cardinal_utxos))
}
