use super::*;
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub cardinal: u64,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  index.update()?;

  let inscription_outputs = index
    .get_inscriptions(None)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut cardinal = 0;
  for (outpoint, amount) in get_unspent_outputs(&options)? {
    if !inscription_outputs.contains(&outpoint) {
      cardinal += amount.to_sat()
    }
  }

  Ok(Box::new(Output { cardinal }))
}
