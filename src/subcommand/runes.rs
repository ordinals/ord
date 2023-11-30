use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, RuneInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneInfo {
  pub burned: u128,
  pub divisibility: u8,
  pub etching: Txid,
  pub height: u32,
  pub id: RuneId,
  pub index: u16,
  pub rune: Rune,
  pub supply: u128,
  pub symbol: Option<char>,
  pub end: Option<u32>,
  pub limit: Option<u128>,
  pub number: u64,
  pub timestamp: DateTime<Utc>,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  ensure!(
    index.has_rune_index(),
    "`ord runes` requires index created with `--index-runes-pre-alpha-i-agree-to-get-rekt` flag",
  );

  index.update()?;

  Ok(Box::new(Output {
    runes: index
      .runes()?
      .into_iter()
      .map(
        |(
          id,
          RuneEntry {
            burned,
            divisibility,
            etching,
            rune,
            supply,
            symbol,
            end,
            limit,
            number,
            timestamp,
          },
        )| {
          (
            rune,
            RuneInfo {
              burned,
              divisibility,
              etching,
              height: id.height,
              id,
              index: id.index,
              end,
              limit,
              number,
              timestamp: crate::timestamp(timestamp),
              rune,
              supply,
              symbol,
            },
          )
        },
      )
      .collect::<BTreeMap<Rune, RuneInfo>>(),
  }))
}
