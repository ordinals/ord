use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Identify {
  #[clap(
    long,
    help = "Find ordinals listed in first column of tab-separated value file <ORDINALS>."
  )]
  ordinals: Option<PathBuf>,
}

impl Identify {
  pub(crate) fn run(&self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.update()?;

    let utxos = list_unspent(&options, &index)?;

    if let Some(path) = &self.ordinals {
      for (output, ordinal, offset, name) in identify_from_tsv(utxos, &fs::read_to_string(path)?)? {
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

fn identify_from_tsv(
  utxos: Vec<(OutPoint, Vec<(u64, u64)>)>,
  tsv: &str,
) -> Result<Vec<(OutPoint, Ordinal, u64, &str)>> {
  let mut needles = Vec::new();

  for (i, line) in tsv.lines().enumerate() {
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    if let Some(value) = line.split("\t").next() {
      let ordinal = Ordinal::from_str(value)
        .map_err(|err| anyhow!("Failed to parse ordinal `{value}` on line {}: {err}", i + 1,))?;

      needles.push((ordinal, value));
    }
  }
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
    let (needle, value) = match needles.get(i) {
      Some(needle) => *needle,
      None => break,
    };

    if needle >= start && needle < end {
      results.push((outpoint, needle, 0, value));
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
    assert_eq!(
      identify_rare(vec![(
        outpoint(1),
        vec![(51 * COIN_VALUE, 100 * COIN_VALUE), (1234, 5678)],
      )]),
      vec![]
    )
  }

  #[test]
  fn identify_one_rare_ordinal() {
    assert_eq!(
      identify_rare(vec![(
        outpoint(1),
        vec![(10, 80), (50 * COIN_VALUE, 100 * COIN_VALUE)],
      )]),
      vec![(outpoint(1), Ordinal(50 * COIN_VALUE), 70, Rarity::Uncommon)]
    )
  }

  #[test]
  fn identify_two_rare_ordinals() {
    assert_eq!(
      identify_rare(vec![(
        outpoint(1),
        vec![(0, 100), (1050000000000000, 1150000000000000)],
      )]),
      vec![
        (outpoint(1), Ordinal(0), 0, Rarity::Mythic),
        (outpoint(1), Ordinal(1050000000000000), 100, Rarity::Epic)
      ]
    )
  }

  #[test]
  fn identify_rare_ordinals_in_different_outpoints() {
    assert_eq!(
      identify_rare(vec![
        (outpoint(1), vec![(50 * COIN_VALUE, 55 * COIN_VALUE)]),
        (outpoint(2), vec![(100 * COIN_VALUE, 111 * COIN_VALUE)],),
      ]),
      vec![
        (outpoint(1), Ordinal(50 * COIN_VALUE), 0, Rarity::Uncommon),
        (outpoint(2), Ordinal(100 * COIN_VALUE), 0, Rarity::Uncommon)
      ]
    )
  }

  // To Do:
  // - two in one range
  // - two from different ranges
  // - tsv out of order
  // - ranges out of order

  #[test]
  fn identify_from_tsv_single() {
    assert_eq!(
      identify_from_tsv(vec![(outpoint(1), vec![(0, 1)])], "0\n").unwrap(),
      vec![(outpoint(1), Ordinal(0), 0, "0"),]
    )
  }

  #[test]
  fn identify_from_tsv_none() {
    assert_eq!(
      identify_from_tsv(vec![(outpoint(1), vec![(0, 1)])], "1\n").unwrap(),
      vec![]
    )
  }

  #[test]
  fn identify_from_tsv_ignores_extra_columns() {
    assert_eq!(
      identify_from_tsv(vec![(outpoint(1), vec![(0, 1)])], "0\t===\n").unwrap(),
      vec![(outpoint(1), Ordinal(0), 0, "0"),]
    )
  }

  #[test]
  fn identify_from_tsv_ignores_empty_lines() {
    assert_eq!(
      identify_from_tsv(vec![(outpoint(1), vec![(0, 1)])], "0\n\n\n").unwrap(),
      vec![(outpoint(1), Ordinal(0), 0, "0"),]
    )
  }

  #[test]
  fn identify_from_tsv_ignores_comments() {
    assert_eq!(
      identify_from_tsv(vec![(outpoint(1), vec![(0, 1)])], "0\n#===\n").unwrap(),
      vec![(outpoint(1), Ordinal(0), 0, "0"),]
    )
  }

  #[test]
  fn parse_error_reports_line_and_value() {
    assert_eq!(
      identify_from_tsv(vec![(outpoint(1), vec![(0, 1)])], "0\n===\n")
        .unwrap_err()
        .to_string(),
      "Failed to parse ordinal `===` on line 2: invalid digit found in string",
    )
  }
}
