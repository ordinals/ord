use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, BTreeMap<OutPoint, u128>>,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  ensure!(
    index.has_rune_index(),
    "`ord balances` requires index created with `--index-runes-pre-alpha-i-agree-to-get-rekt` flag",
  );

  index.update()?;

  Ok(Box::new(Output {
    runes: index.get_rune_balance_map()?,
  }))
}
