use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Identify {
  #[clap(
    long,
    help = "Find ordinals from first column of file of tab-separated values <ORDINALS>."
  )]
  ordinals: Option<PathBuf>,
}

impl Identify {
  pub(crate) fn run(&self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.update()?;

    let utxos = list_unspent(&options, &index)?;

    if let Some(path) = &self.ordinals {
      for (output, ordinal, offset, name) in identify_from(utxos, &fs::read_to_string(path)?)? {
        println!("{output}\t{ordinal}\t{offset}\t{name}");
      }
    } else {
      for (output, ordinal, offset, rarity) in identify_rare(utxos) {
        println!("{output}\t{ordinal}\t{offset}\t{rarity}");
      }
    }

    Ok(())
  }
}

fn identify_rare(utxos: Vec<(OutPoint, Vec<(u64, u64)>)>) -> Vec<(OutPoint, Ordinal, u64, Rarity)> {
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

fn identify_from(
  utxos: Vec<(OutPoint, Vec<(u64, u64)>)>,
  tsv: &str,
) -> Result<Vec<(OutPoint, Ordinal, u64, &str)>> {
  // To Do:
  // - test parsing from multiple representations
  // - test line reported line number is correct
  // - test reported string is correct
  // - test no ordinals found
  // - test first ordinal
  // - skip empty lines
  // - skip comments

  let mut needles = tsv
    .lines()
    .enumerate()
    .flat_map(|(i, line)| {
      line.split("\t").next().map(|column| {
        Ordinal::from_str(column)
          .map(|ordinal| (ordinal, column))
          .map_err(|err| {
            anyhow!(
              "Failed to parse ordinal `{column}` on line {}: {err}",
              i + 1,
            )
          })
      })
    })
    .collect::<Result<Vec<(Ordinal, &str)>>>()?;
  needles.sort();

  let mut haystacks = utxos
    .into_iter()
    .flat_map(|(outpoint, ranges)| {
      ranges
        .into_iter()
        .map(move |(start, end)| (start, end, outpoint))
    })
    .collect::<Vec<(u64, u64, OutPoint)>>();
  haystacks.sort();

  let mut i = 0;
  let mut results = Vec::new();
  for (start, end, outpoint) in haystacks {
    let (needle, column) = match needles.get(i) {
      Some(needle) => *needle,
      None => break,
    };

    if needle >= start && needle < end {
      results.push((outpoint, needle, 0, column));
    }

    if needle >= end {
      i += 1;
    }
  }

  Ok(results)
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
    assert_eq!(identify_rare(utxos), vec![])
  }

  #[test]
  fn identify_one_rare_ordinal() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(10, 80), (50 * COIN_VALUE, 100 * COIN_VALUE)],
    )];
    assert_eq!(
      identify_rare(utxos),
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
      identify_rare(utxos),
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
      identify_rare(utxos),
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
