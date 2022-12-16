use super::*;

#[derive(PartialEq, Debug)]
pub(crate) struct Degree {
  pub(crate) hour: u64,
  pub(crate) minute: u64,
  pub(crate) second: u64,
  pub(crate) third: u64,
}

impl Display for Degree {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}°{}′{}″{}‴",
      self.hour, self.minute, self.second, self.third
    )
  }
}

impl From<Sat> for Degree {
  fn from(sat: Sat) -> Self {
    let height = sat.height().n();
    Degree {
      hour: height / (CYCLE_EPOCHS * SUBSIDY_HALVING_INTERVAL),
      minute: height % SUBSIDY_HALVING_INTERVAL,
      second: height % DIFFCHANGE_INTERVAL,
      third: sat.third(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn case(sat: u64, hour: u64, minute: u64, second: u64, third: u64) {
    assert_eq!(
      Degree::from(Sat(sat)),
      Degree {
        hour,
        minute,
        second,
        third,
      }
    );
  }

  #[test]
  fn from() {
    case(0, 0, 0, 0, 0);
    case(1, 0, 0, 0, 1);
    case(5_000_000_000, 0, 1, 1, 0);
    case(
      5_000_000_000 * DIFFCHANGE_INTERVAL,
      0,
      DIFFCHANGE_INTERVAL,
      0,
      0,
    );
    case(5_000_000_000 * SUBSIDY_HALVING_INTERVAL, 0, 0, 336, 0);
    case(
      5_000_000_000 * SUBSIDY_HALVING_INTERVAL
        + 2_500_000_000 * SUBSIDY_HALVING_INTERVAL
        + 1_250_000_000 * SUBSIDY_HALVING_INTERVAL
        + 625_000_000 * SUBSIDY_HALVING_INTERVAL
        + 312_500_000 * SUBSIDY_HALVING_INTERVAL
        + 156_250_000 * SUBSIDY_HALVING_INTERVAL,
      1,
      0,
      0,
      0,
    );
  }
}
