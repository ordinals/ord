use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, BTreeMap<OutPoint, u128>>,
}

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let index = Index::open(&settings)?;

  ensure!(
    index.has_rune_index(),
    "`ord balances` requires index created with `--index-runes` flag",
  );

  index.update()?;

  Ok(Some(Box::new(Output {
    runes: index.get_rune_balance_map()?,
  })))
}
