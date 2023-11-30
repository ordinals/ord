use {super::*, crate::wallet::Wallet, std::collections::BTreeSet};

#[derive(Serialize, Deserialize)]
pub struct CardinalUtxo {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  index.update()?;

  let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;

  let inscribed_utxos = index
    .get_inscriptions(&unspent_outputs)?
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
