use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.index()?;

  let utxos = list_unspent(&options, &index)?;

  for (output, ordinal, offset, rarity) in identify(utxos) {
    println!("{output}\t{ordinal}\t{offset}\t{rarity}");
  }

  Ok(())
}

fn identify(utxos: Vec<(OutPoint, Vec<(u64, u64)>)>) -> Vec<(OutPoint, Ordinal, u64, Rarity)> {
  utxos
    .into_iter()
    .flat_map(|(outpoint, ordinal_ranges)| {
      let mut offset = 0;
      ordinal_ranges.into_iter().filter_map(move |(start, end)| {
        let ordinal = Ordinal(start);
        let rarity = ordinal.rarity();
        let start_offset = offset;
        offset += end - start;
        if rarity > Rarity::Common {
          Some((outpoint, ordinal, start_offset, rarity))
        } else {
          None
        }
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn identify_no_rare_ordinals() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(51 * COIN_VALUE, 100 * COIN_VALUE), (1234, 5678)],
    )];
    assert_eq!(identify(utxos), vec![])
  }

  #[test]
  fn identify_one_rare_ordinal() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(10, 80), (50 * COIN_VALUE, 100 * COIN_VALUE)],
    )];
    assert_eq!(
      identify(utxos),
      vec![(
        OutPoint::null(),
        Ordinal(50 * COIN_VALUE),
        70,
        Rarity::Uncommon
      )]
    )
  }

  #[test]
  fn identify_two_rare_ordinals() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(0, 100), (1050000000000000, 1150000000000000)],
    )];
    assert_eq!(
      identify(utxos),
      vec![
        (OutPoint::null(), Ordinal(0), 0, Rarity::Mythic),
        (
          OutPoint::null(),
          Ordinal(1050000000000000),
          100,
          Rarity::Epic
        )
      ]
    )
  }

  #[test]
  fn identify_rare_ordinals_in_different_outpoints() {
    let utxos = vec![
      (OutPoint::null(), vec![(50 * COIN_VALUE, 55 * COIN_VALUE)]),
      (
        OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
          .unwrap(),
        vec![(100 * COIN_VALUE, 111 * COIN_VALUE)],
      ),
    ];
    assert_eq!(
      identify(utxos),
      vec![
        (
          OutPoint::null(),
          Ordinal(50 * COIN_VALUE),
          0,
          Rarity::Uncommon
        ),
        (
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          Ordinal(100 * COIN_VALUE),
          0,
          Rarity::Uncommon
        )
      ]
    )
  }
}
