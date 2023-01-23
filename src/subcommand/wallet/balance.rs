use super::*;
use std::collections::BTreeSet;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let inscription_outputs = index
    .get_inscriptions(None)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let cardinal_balance = get_unspent_outputs(&options)?
    .iter()
    .map(|(outpoint, amount)| {
      if inscription_outputs.contains(outpoint) {
        0
      } else {
        amount.to_sat()
      }
    })
    .sum::<u64>();

  println!("{}", cardinal_balance);

  Ok(())
}
