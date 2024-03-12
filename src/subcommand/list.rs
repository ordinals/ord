use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  #[arg(help = "List sats in <OUTPOINT>.")]
  outpoint: OutPoint,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub ranges: Option<Vec<Range>>,
  pub spent: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Range {
  pub end: u64,
  pub name: String,
  pub offset: u64,
  pub rarity: Rarity,
  pub size: u64,
  pub start: u64,
}

impl List {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    if !index.has_sat_index() {
      bail!("list requires index created with `--index-sats` flag");
    }

    index.update()?;

    ensure! {
      index.is_output_in_active_chain(self.outpoint)?,
      "output not found"
    }

    let ranges = index.list(self.outpoint)?;

    let spent = index.is_output_spent(self.outpoint)?;

    Ok(Some(Box::new(Output {
      spent,
      ranges: ranges.map(output_ranges),
    })))
  }
}

fn output_ranges(ranges: Vec<(u64, u64)>) -> Vec<Range> {
  let mut offset = 0;
  ranges
    .into_iter()
    .map(|(start, end)| {
      let size = end - start;
      let output = Range {
        end,
        name: Sat(start).name(),
        offset,
        rarity: Sat(start).rarity(),
        size,
        start,
      };

      offset += size;

      output
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_ranges() {
    assert_eq!(
      output_ranges(vec![
        (50 * COIN_VALUE, 55 * COIN_VALUE),
        (10, 100),
        (1050000000000000, 1150000000000000),
      ]),
      vec![
        Range {
          end: 55 * COIN_VALUE,
          name: "nvtcsezkbth".to_string(),
          offset: 0,
          rarity: Rarity::Uncommon,
          size: 5 * COIN_VALUE,
          start: 50 * COIN_VALUE,
        },
        Range {
          end: 100,
          name: "nvtdijuwxlf".to_string(),
          offset: 5 * COIN_VALUE,
          rarity: Rarity::Common,
          size: 90,
          start: 10,
        },
        Range {
          end: 1150000000000000,
          name: "gkjbdrhkfqf".to_string(),
          offset: 5 * COIN_VALUE + 90,
          rarity: Rarity::Epic,
          size: 100000000000000,
          start: 1050000000000000,
        }
      ]
    )
  }
}
