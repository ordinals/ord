use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub frozen_runes: BTreeMap<SpacedRune, BTreeMap<OutPoint, Pile>>,
}

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let index = Index::open(&settings)?;

  ensure!(
    index.has_rune_index(),
    "`ord frozen` requires index created with `--index-runes` flag",
  );

  index.update()?;

  Ok(Some(Box::new(Output {
    frozen_runes: index.get_frozen_rune_balance_map()?,
  })))
}
