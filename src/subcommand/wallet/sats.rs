use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Sats {
  #[arg(
    long,
    help = "Find satoshis listed in first column of tab-separated value file <TSV>."
  )]
  tsv: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct OutputTsv {
  pub found: BTreeMap<String, SatPoint>,
  pub lost: BTreeSet<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OutputRare {
  pub sat: Sat,
  pub output: OutPoint,
  pub offset: u64,
  pub rarity: Rarity,
}

impl Sats {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_sat_index(),
      "sats requires index created with `--index-sats` flag"
    );

    let haystacks = wallet.get_output_sat_ranges()?;

    if let Some(path) = &self.tsv {
      let tsv = fs::read_to_string(path)
        .with_context(|| format!("I/O error reading `{}`", path.display()))?;

      let needles = Self::needles(&tsv)?;

      let found = Self::find(&needles, &haystacks);

      let lost = needles
        .into_iter()
        .filter(|(_sat, value)| !found.contains_key(*value))
        .map(|(_sat, value)| value.into())
        .collect();

      Ok(Some(Box::new(OutputTsv { found, lost })))
    } else {
      let mut output = Vec::new();
      for (outpoint, sat, offset, rarity) in Self::rare_sats(haystacks) {
        output.push(OutputRare {
          sat,
          output: outpoint,
          offset,
          rarity,
        });
      }
      Ok(Some(Box::new(output)))
    }
  }

  fn find(
    needles: &[(Sat, &str)],
    ranges: &[(OutPoint, Vec<(u64, u64)>)],
  ) -> BTreeMap<String, SatPoint> {
    let mut haystacks = Vec::new();

    for (outpoint, ranges) in ranges {
      let mut offset = 0;
      for (start, end) in ranges {
        haystacks.push((start, end, offset, outpoint));
        offset += end - start;
      }
    }

    haystacks.sort_by_key(|(start, _, _, _)| *start);

    let mut i = 0;
    let mut j = 0;
    let mut results = BTreeMap::new();
    while i < needles.len() && j < haystacks.len() {
      let (needle, value) = needles[i];
      let (&start, &end, offset, outpoint) = haystacks[j];

      if needle >= start && needle < end {
        results.insert(
          value.into(),
          SatPoint {
            outpoint: *outpoint,
            offset: offset + needle.0 - start,
          },
        );
      }

      if needle >= end {
        j += 1;
      } else {
        i += 1;
      }
    }

    results
  }

  fn needles(tsv: &str) -> Result<Vec<(Sat, &str)>> {
    let mut needles = tsv
      .lines()
      .enumerate()
      .filter(|(_i, line)| !line.starts_with('#') && !line.is_empty())
      .filter_map(|(i, line)| {
        line.split('\t').next().map(|value| {
          Sat::from_str(value).map(|sat| (sat, value)).map_err(|err| {
            anyhow!(
              "failed to parse sat from string \"{value}\" on line {}: {err}",
              i + 1,
            )
          })
        })
      })
      .collect::<Result<Vec<(Sat, &str)>>>()?;

    needles.sort();

    Ok(needles)
  }

  fn rare_sats(haystacks: Vec<(OutPoint, Vec<(u64, u64)>)>) -> Vec<(OutPoint, Sat, u64, Rarity)> {
    haystacks
      .into_iter()
      .flat_map(|(outpoint, sat_ranges)| {
        let mut offset = 0;
        sat_ranges.into_iter().filter_map(move |(start, end)| {
          let sat = Sat(start);
          let rarity = sat.rarity();
          let start_offset = offset;
          offset += end - start;
          if rarity > Rarity::Common {
            Some((outpoint, sat, start_offset, rarity))
          } else {
            None
          }
        })
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn identify_no_rare_sats() {
    assert_eq!(
      Sats::rare_sats(vec![(
        outpoint(1),
        vec![(51 * COIN_VALUE, 100 * COIN_VALUE), (1234, 5678)],
      )]),
      Vec::new()
    )
  }

  #[test]
  fn identify_one_rare_sat() {
    assert_eq!(
      Sats::rare_sats(vec![(
        outpoint(1),
        vec![(10, 80), (50 * COIN_VALUE, 100 * COIN_VALUE)],
      )]),
      vec![(outpoint(1), Sat(50 * COIN_VALUE), 70, Rarity::Uncommon)]
    )
  }

  #[test]
  fn identify_two_rare_sats() {
    assert_eq!(
      Sats::rare_sats(vec![(
        outpoint(1),
        vec![(0, 100), (1050000000000000, 1150000000000000)],
      )]),
      vec![
        (outpoint(1), Sat(0), 0, Rarity::Mythic),
        (outpoint(1), Sat(1050000000000000), 100, Rarity::Epic)
      ]
    )
  }

  #[test]
  fn identify_rare_sats_in_different_outpoints() {
    assert_eq!(
      Sats::rare_sats(vec![
        (outpoint(1), vec![(50 * COIN_VALUE, 55 * COIN_VALUE)]),
        (outpoint(2), vec![(100 * COIN_VALUE, 111 * COIN_VALUE)],),
      ]),
      vec![
        (outpoint(1), Sat(50 * COIN_VALUE), 0, Rarity::Uncommon),
        (outpoint(2), Sat(100 * COIN_VALUE), 0, Rarity::Uncommon)
      ]
    )
  }

  #[track_caller]
  fn case(tsv: &str, haystacks: &[(OutPoint, Vec<(u64, u64)>)], expected: &[(&str, SatPoint)]) {
    assert_eq!(
      Sats::find(&Sats::needles(tsv).unwrap(), haystacks),
      expected
        .iter()
        .map(|(sat, satpoint)| (sat.to_string(), *satpoint))
        .collect()
    );
  }

  #[test]
  fn tsv() {
    case("1\n", &[(outpoint(1), vec![(0, 1)])], &[]);
  }

  #[test]
  fn identify_from_tsv_single() {
    case(
      "0\n",
      &[(outpoint(1), vec![(0, 1)])],
      &[("0", satpoint(1, 0))],
    );
  }

  #[test]
  fn identify_from_tsv_two_in_one_range() {
    case(
      "0\n1\n",
      &[(outpoint(1), vec![(0, 2)])],
      &[("0", satpoint(1, 0)), ("1", satpoint(1, 1))],
    );
  }

  #[test]
  fn identify_from_tsv_out_of_order_tsv() {
    case(
      "1\n0\n",
      &[(outpoint(1), vec![(0, 2)])],
      &[("0", satpoint(1, 0)), ("1", satpoint(1, 1))],
    );
  }

  #[test]
  fn identify_from_tsv_out_of_order_ranges() {
    case(
      "1\n0\n",
      &[(outpoint(1), vec![(1, 2), (0, 1)])],
      &[("0", satpoint(1, 1)), ("1", satpoint(1, 0))],
    );
  }

  #[test]
  fn identify_from_tsv_two_in_two_ranges() {
    case(
      "0\n1\n",
      &[(outpoint(1), vec![(0, 1), (1, 2)])],
      &[("0", satpoint(1, 0)), ("1", satpoint(1, 1))],
    )
  }

  #[test]
  fn identify_from_tsv_two_in_two_outputs() {
    case(
      "0\n1\n",
      &[(outpoint(1), vec![(0, 1)]), (outpoint(2), vec![(1, 2)])],
      &[("0", satpoint(1, 0)), ("1", satpoint(2, 0))],
    );
  }

  #[test]
  fn identify_from_tsv_ignores_extra_columns() {
    case(
      "0\t===\n",
      &[(outpoint(1), vec![(0, 1)])],
      &[("0", satpoint(1, 0))],
    );
  }

  #[test]
  fn identify_from_tsv_ignores_empty_lines() {
    case(
      "0\n\n\n",
      &[(outpoint(1), vec![(0, 1)])],
      &[("0", satpoint(1, 0))],
    );
  }

  #[test]
  fn identify_from_tsv_ignores_comments() {
    case(
      "0\n#===\n",
      &[(outpoint(1), vec![(0, 1)])],
      &[("0", satpoint(1, 0))],
    );
  }

  #[test]
  fn parse_error_reports_line_and_value() {
    assert_eq!(
      Sats::needles("0\n===\n")
        .unwrap_err()
        .to_string(),
      "failed to parse sat from string \"===\" on line 2: failed to parse sat `===`: invalid integer: invalid digit found in string",
    );
  }
}
