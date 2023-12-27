use {super::*, std::collections::BTreeSet};

#[derive(Serialize, Deserialize)]
pub struct CardinalUtxo {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(no_sync: bool, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  if !no_sync {
    index.update()?;
  }

  let wallet_client = options.bitcoin_rpc_client_for_wallet_command(options.wallet.clone())?;

  let unspent_outputs = Wallet::get_unspent_outputs(&wallet_client, &index)?;

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
