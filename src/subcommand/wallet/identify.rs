use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::index(&options)?;

  let mut ordinals = Purse::load(&options)?
    .wallet
    .list_unspent()?
    .into_iter()
    .map(|utxo| {
      index.list(utxo.outpoint).and_then(|list| match list {
        Some(List::Unspent(ranges)) => Ok(
          ranges
            .into_iter()
            .map(|(start, _end)| Ordinal(start))
            .filter(|ordinal| ordinal.rarity() > Rarity::Common)
            .map(|ordinal| (ordinal, utxo.outpoint))
            .collect(),
        ),
        Some(List::Spent(txid)) => Err(anyhow!(
          "UTXO {} unspent in wallet but spent in index by transaction {txid}",
          utxo.outpoint
        )),
        None => Ok(Vec::new()),
      })
    })
    .collect::<Result<Vec<Vec<(Ordinal, OutPoint)>>, _>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<(Ordinal, OutPoint)>>();

  ordinals.sort_by(|(ordinal_a, _), (ordinal_b, _)| ordinal_a.cmp(ordinal_b));

  for (ordinal, outpoint) in ordinals {
    println!("{ordinal} {} {outpoint}", ordinal.rarity());
  }

  Ok(())
}
