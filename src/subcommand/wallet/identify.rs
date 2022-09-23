use super::*;

pub(crate) fn run(options: Options) -> Result {
  let purse = Purse::load(&options)?;

  let index = Index::open(&options)?;

  index.index()?;

  let mut ordinals = purse
    .wallet
    .list_unspent()?
    .into_iter()
    .map(|utxo| {
      purse
        .special_ordinals(&index, utxo.outpoint)
        .map(|ordinals| {
          ordinals
            .into_iter()
            .map(|ordinal| (ordinal, utxo.outpoint))
            .collect::<Vec<(Ordinal, OutPoint)>>()
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
