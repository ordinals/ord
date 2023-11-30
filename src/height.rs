use super::*;

#[derive(Copy, Clone, Debug, Display, FromStr, Ord, Eq, Serialize, PartialEq, PartialOrd)]
pub(crate) struct Height(pub(crate) u32);

impl Height {
  pub(crate) fn n(self) -> u32 {
    self.0
  }

  pub(crate) fn subsidy(self) -> u64 {
    Epoch::from(self).subsidy()
  }

  pub(crate) fn starting_sat(self) -> Sat {
    let epoch = Epoch::from(self);
    let epoch_starting_sat = epoch.starting_sat();
    let epoch_starting_height = epoch.starting_height();
    epoch_starting_sat + u64::from(self.n() - epoch_starting_height.n()) * epoch.subsidy()
  }

  pub(crate) fn period_offset(self) -> u32 {
    self.0 % DIFFCHANGE_INTERVAL
  }
}

impl Add<u32> for Height {
  type Output = Self;

  fn add(self, other: u32) -> Height {
    Self(self.0 + other)
  }
}

impl Sub<u32> for Height {
  type Output = Self;

  fn sub(self, other: u32) -> Height {
    Self(self.0 - other)
  }
}

impl PartialEq<u32> for Height {
  fn eq(&self, other: &u32) -> bool {
    self.0 == *other
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn n() {
    assert_eq!(Height(0).n(), 0);
    assert_eq!(Height(1).n(), 1);
  }

  #[test]
  fn add() {
    assert_eq!(Height(0) + 1, 1);
    assert_eq!(Height(1) + 100, 101);
  }

  #[test]
  fn sub() {
    assert_eq!(Height(1) - 1, 0);
    assert_eq!(Height(100) - 50, 50);
  }

  #[test]
  fn eq() {
    assert_eq!(Height(0), 0);
    assert_eq!(Height(100), 100);
  }

  #[test]
  fn from_str() {
    assert_eq!("0".parse::<Height>().unwrap(), 0);
    assert!("foo".parse::<Height>().is_err());
  }

  #[test]
  fn subsidy() {
    assert_eq!(Height(0).subsidy(), 5000000000);
    assert_eq!(Height(1).subsidy(), 5000000000);
    assert_eq!(Height(SUBSIDY_HALVING_INTERVAL - 1).subsidy(), 5000000000);
    assert_eq!(Height(SUBSIDY_HALVING_INTERVAL).subsidy(), 2500000000);
    assert_eq!(Height(SUBSIDY_HALVING_INTERVAL + 1).subsidy(), 2500000000);
  }

  #[test]
  fn starting_sat() {
    assert_eq!(Height(0).starting_sat(), 0);
    assert_eq!(Height(1).starting_sat(), 5000000000);
    assert_eq!(
      Height(SUBSIDY_HALVING_INTERVAL - 1).starting_sat(),
      (u64::from(SUBSIDY_HALVING_INTERVAL) - 1) * 5000000000
    );
    assert_eq!(
      Height(SUBSIDY_HALVING_INTERVAL).starting_sat(),
      u64::from(SUBSIDY_HALVING_INTERVAL) * 5000000000
    );
    assert_eq!(
      Height(SUBSIDY_HALVING_INTERVAL + 1).starting_sat(),
      u64::from(SUBSIDY_HALVING_INTERVAL) * 5000000000 + 2500000000
    );
    assert_eq!(
      Height(u32::max_value()).starting_sat(),
      *Epoch::STARTING_SATS.last().unwrap()
    );
  }

  #[test]
  fn period_offset() {
    assert_eq!(Height(0).period_offset(), 0);
    assert_eq!(Height(1).period_offset(), 1);
    assert_eq!(Height(DIFFCHANGE_INTERVAL - 1).period_offset(), 2015);
    assert_eq!(Height(DIFFCHANGE_INTERVAL).period_offset(), 0);
    assert_eq!(Height(DIFFCHANGE_INTERVAL + 1).period_offset(), 1);
  }
}
