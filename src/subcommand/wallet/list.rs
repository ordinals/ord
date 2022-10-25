use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let utxos = list_unspent(&options, &index)?;

  for (output, start, size, rarity, name) in list(utxos) {
    println!("{output}\t{start}\t{size}\t{rarity}\t{name}");
  }

  Ok(())
}

fn list(utxos: Vec<(OutPoint, Vec<(u64, u64)>)>) -> Vec<(OutPoint, u64, u64, Rarity, String)> {
  utxos
    .into_iter()
    .flat_map(|(output, ordinal_ranges)| {
      ordinal_ranges.into_iter().map(move |(start, end)| {
        let ordinal = Ordinal(start);
        let rarity = ordinal.rarity();
        let name = ordinal.name();
        let size = end - start;
        (output, start, size, rarity, name)
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_ranges() {
    let utxos = vec![
      (
        OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
          .unwrap(),
        vec![(50 * COIN_VALUE, 55 * COIN_VALUE)],
      ),
      (
        OutPoint::null(),
        vec![(10, 100), (1050000000000000, 1150000000000000)],
      ),
    ];
    assert_eq!(
      list(utxos),
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
          OutPoint::null(),
          10,
          90,
          Rarity::Common,
          "nvtdijuwxlf".to_string()
        ),
        (
          OutPoint::null(),
          1050000000000000,
          100000000000000,
          Rarity::Epic,
          "gkjbdrhkfqf".to_string()
        )
      ]
    )
  }
}
