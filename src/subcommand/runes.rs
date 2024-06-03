use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, RuneInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneInfo {
  pub block: u64,
  pub burned: u128,
  pub divisibility: u8,
  pub etching: Txid,
  pub id: RuneId,
  pub mints: u128,
  pub number: u64,
  pub premine: u128,
  pub rune: SpacedRune,
  pub supply: u128,
  pub symbol: Option<char>,
  pub terms: Option<Terms>,
  pub timestamp: DateTime<Utc>,
  pub turbo: bool,
  pub tx: u32,
}

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let index = Index::open(&settings)?;

  ensure!(
    index.has_rune_index(),
    "`ord runes` requires index created with `--index-runes` flag",
  );

  index.update()?;

  Ok(Some(Box::new(Output {
    runes: index
      .runes()?
      .into_iter()
      .map(
        |(
          id,
          entry @ RuneEntry {
            block,
            burned,
            divisibility,
            etching,
            mints,
            number,
            premine,
            spaced_rune,
            symbol,
            terms,
            timestamp,
            turbo,
          },
        )| {
          (
            spaced_rune.rune,
            RuneInfo {
              block,
              burned,
              divisibility,
              etching,
              id,
              mints,
              number,
              premine,
              rune: spaced_rune,
              supply: entry.supply(),
              symbol,
              terms,
              timestamp: crate::timestamp(timestamp),
              turbo,
              tx: id.tx,
            },
          )
        },
      )
      .collect::<BTreeMap<Rune, RuneInfo>>(),
  })))
}
