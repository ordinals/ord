use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::index(&options)?;

  let mut ordinals = Purse::load(&options)?
    .wallet
    .list_unspent()?
    .into_iter()
    .map(|utxo| {
      index.list(utxo.outpoint).and_then(|list| match list {
        Some(List::Unspent(ranges)) => Ok((
          utxo.clone(),
          ranges
            .iter()
            .map(|(start, _end)| Ordinal(*start))
            .filter(|ordinal| ordinal.rarity() > Rarity::Common)
            .collect(),
        )),
        Some(List::Spent(txid)) => Err(anyhow!(
          "UTXO {} unspent in wallet but spent in index by transaction {txid}",
          utxo.outpoint
        )),
        None => Ok((utxo.clone(), Vec::new())),
      })
    })
    .collect::<Result<Vec<(LocalUtxo, Vec<Ordinal>)>, _>>()?;

  ordinals.sort_by(|a, b| a.1.cmp(&b.1));

  for (utxo, ordinals) in ordinals {
    for ordinal in ordinals {
      println!("{ordinal} {} {}", ordinal.rarity(), utxo.outpoint);
    }
  }

  Ok(())
}
