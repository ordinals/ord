use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, RuneInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneInfo {
  pub burned: u128,
  pub deadline: Option<u32>,
  pub divisibility: u8,
  pub end: Option<u32>,
  pub etching: Txid,
  pub height: u32,
  pub id: RuneId,
  pub index: u16,
  pub limit: Option<u128>,
  pub number: u64,
  pub rune: Rune,
  pub spacers: u32,
  pub supply: u128,
  pub symbol: Option<char>,
  pub timestamp: DateTime<Utc>,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  ensure!(
    index.has_rune_index(),
    "`ord runes` requires index created with `--index-runes` flag",
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
            deadline,
            divisibility,
            end,
            etching,
            limit,
            number,
            rune,
            spacers,
            supply,
            symbol,
            timestamp,
          },
        )| {
          (
            rune,
            RuneInfo {
              burned,
              deadline,
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
              spacers,
              supply,
              symbol,
            },
          )
        },
      )
      .collect::<BTreeMap<Rune, RuneInfo>>(),
  }))
}
