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

impl From<Ordinal> for Degree {
  fn from(ordinal: Ordinal) -> Self {
    let height = ordinal.height().n();
    Degree {
      hour: height / (CYCLE_EPOCHS * Epoch::BLOCKS),
      minute: height % Epoch::BLOCKS,
      second: height % PERIOD_BLOCKS,
      third: ordinal.third(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn case(ordinal: u64, hour: u64, minute: u64, second: u64, third: u64) {
    assert_eq!(
      Degree::from(Ordinal(ordinal)),
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
    case(5_000_000_000 * 2016, 0, 2016, 0, 0);
    case(5_000_000_000 * 210_000, 0, 0, 336, 0);
    case(
      5_000_000_000 * 210_000
        + 2_500_000_000 * 210_000
        + 1_250_000_000 * 210_000
        + 625_000_000 * 210_000
        + 312_500_000 * 210_000
        + 156_250_000 * 210_000,
      1,
      0,
      0,
      0,
    );
  }
}
