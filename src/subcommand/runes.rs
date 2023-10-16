use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, RuneInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneInfo {
  burned: u128,
  divisibility: u8,
  etching: Txid,
  height: u32,
  id: u128,
  index: u16,
  rune: Rune,
  supply: u128,
  symbol: Option<char>,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  if !index.has_rune_index()? {
    todo!();
  }

  index.update()?;

  Ok(Box::new(Output {
    runes: index
      .runes()?
      .unwrap()
      .into_iter()
      .map(|(id, entry)| {
        let RuneEntry {
          burned,
          divisibility,
          etching,
          rune,
          supply,
          symbol,
        } = entry;

        (
          rune,
          RuneInfo {
            burned,
            divisibility,
            etching,
            height: id.height,
            id: id.into(),
            index: id.index,
            rune,
            supply,
            symbol,
          },
        )
      })
      .collect::<BTreeMap<Rune, RuneInfo>>(),
  }))
}
