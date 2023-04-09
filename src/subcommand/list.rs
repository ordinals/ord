use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  #[clap(help = "List sats in <OUTPOINT>.")]
  outpoint: OutPoint,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub start: u64,
  pub end: u64,
  pub size: u64,
  pub offset: u64,
  pub rarity: Rarity,
  pub name: String,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    match index.list(self.outpoint)? {
      Some(crate::index::List::Unspent(ranges)) => {
        let mut outputs = Vec::new();
        let mut offset = 0;
        for (output, start, end, size, rarity, name) in list(self.outpoint, ranges) {
          outputs.push(Output {
            output,
            start,
            end,
            size,
            offset,
            rarity,
            name,
          });
          offset += size;
        }

        print_json(outputs)?;

        Ok(())
      }
      Some(crate::index::List::Spent) => Err(anyhow!("output spent.")),
      None => Err(anyhow!("output not found")),
    }
  }
}

fn list(
  outpoint: OutPoint,
  ranges: Vec<(u64, u64)>,
) -> Vec<(OutPoint, u64, u64, u64, Rarity, String)> {
  ranges
    .into_iter()
    .map(|(start, end)| {
      let size = end - start;
      let rarity = Sat(start).rarity();
      let name = Sat(start).name();

      (outpoint, start, end, size, rarity, name)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_ranges() {
    let outpoint =
      OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
        .unwrap();
    let ranges = vec![
      (50 * COIN_VALUE, 55 * COIN_VALUE),
      (10, 100),
      (1050000000000000, 1150000000000000),
    ];
    assert_eq!(
      list(outpoint, ranges),
      vec![
        (
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          50 * COIN_VALUE,
          55 * COIN_VALUE,
          5 * COIN_VALUE,
          Rarity::Uncommon,
          "nvtcsezkbth".to_string()
        ),
        (
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          10,
          100,
          90,
          Rarity::Common,
          "nvtdijuwxlf".to_string()
        ),
        (
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          1050000000000000,
          1150000000000000,
          100000000000000,
          Rarity::Epic,
          "gkjbdrhkfqf".to_string()
        )
      ]
    )
  }
}
