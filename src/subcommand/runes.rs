use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub runes: BTreeMap<Rune, RuneInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneInfo {
  pub block: u32,
  pub burned: u128,
  pub divisibility: u8,
  pub etching: Txid,
  pub id: RuneId,
  pub mint: Option<MintEntry>,
  pub mints: u64,
  pub number: u64,
  pub rune: Rune,
  pub spacers: u32,
  pub supply: u128,
  pub symbol: Option<char>,
  pub timestamp: DateTime<Utc>,
  pub tx: u16,
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
          RuneEntry {
            burned,
            divisibility,
            etching,
            mint,
            mints,
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
              block: id.block,
              burned,
              divisibility,
              etching,
              id,
              mint,
              mints,
              number,
              rune,
              spacers,
              supply,
              symbol,
              timestamp: crate::timestamp(timestamp),
              tx: id.tx,
            },
          )
        },
      )
      .collect::<BTreeMap<Rune, RuneInfo>>(),
  })))
}
