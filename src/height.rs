use super::*;

#[derive(Copy, Clone, Debug, Display)]
pub(crate) struct Height(pub(crate) u64);

impl Height {
  pub(crate) fn n(self) -> u64 {
    self.0
  }

  pub(crate) fn subsidy(self) -> u64 {
    Epoch::from(self).subsidy()
  }
}

impl Add<u64> for Height {
  type Output = Self;

  fn add(self, other: u64) -> Height {
    Self(self.0 + other)
  }
}

impl Sub<u64> for Height {
  type Output = Self;

  fn sub(self, other: u64) -> Height {
    Self(self.0 - other)
  }
}

impl PartialEq<u64> for Height {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl FromStr for Height {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.parse::<u64>()?))
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
    assert_eq!(Height(210000 - 1).subsidy(), 5000000000);
    assert_eq!(Height(210000).subsidy(), 2500000000);
    assert_eq!(Height(210000 + 1).subsidy(), 2500000000);
  }
}
