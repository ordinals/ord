use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  #[clap(help = "List ordinal ranges in <OUTPOINT>.")]
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    match index.list(self.outpoint)? {
      Some(crate::index::List::Unspent(ranges)) => {
        for (output, start, size, rarity, name) in list(self.outpoint, ranges) {
          println!("{output}\t{start}\t{size}\t{rarity}\t{name}");
        }

        Ok(())
      }
      Some(crate::index::List::Spent) => Err(anyhow!("output spent.")),
      None => Err(anyhow!("output not found")),
    }
  }
}

fn list(outpoint: OutPoint, ranges: Vec<(u64, u64)>) -> Vec<(OutPoint, u64, u64, Rarity, String)> {
  ranges
    .into_iter()
    .map(|(start, end)| {
      let size = end - start;
      let rarity = Ordinal(start).rarity();
      let name = Ordinal(start).name();

      (outpoint, start, size, rarity, name)
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
          5 * COIN_VALUE,
          Rarity::Uncommon,
          "nvtcsezkbth".to_string()
        ),
        (
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          10,
          90,
          Rarity::Common,
          "nvtdijuwxlf".to_string()
        ),
        (
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          1050000000000000,
          100000000000000,
          Rarity::Epic,
          "gkjbdrhkfqf".to_string()
        )
      ]
    )
  }
}
