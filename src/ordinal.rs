use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Ord, PartialOrd)]
pub(crate) struct Ordinal(u64);

impl Ordinal {
  pub(crate) const LAST: Ordinal = Self::new(Self::SUPPLY - 1);

  pub(crate) const SUPPLY: u64 = 2099999997690000;

  pub(crate) const fn new(inner: u64) -> Self {
    assert!(inner < Self::SUPPLY);
    Self(inner)
  }

  pub(crate) fn new_checked(inner: u64) -> Option<Self> {
    if inner < Self::SUPPLY {
      Some(Self(inner))
    } else {
      None
    }
  }

  pub(crate) fn height(self) -> Height {
    self.epoch().starting_height() + self.epoch_position() / self.epoch().subsidy()
  }

  pub(crate) fn epoch(self) -> Epoch {
    self.into()
  }

  pub(crate) fn subsidy_position(self) -> u64 {
    self.epoch_position() % self.epoch().subsidy()
  }

  pub(crate) fn epoch_position(self) -> u64 {
    self.0 - self.epoch().starting_ordinal().unwrap().0
  }

  pub(crate) fn name(self) -> String {
    let mut x = Self::SUPPLY - self.0 - 1;
    let mut name = String::new();
    while x > 0 {
      name.push(
        "abcdefghijklmnopqrstuvwxyz"
          .chars()
          .nth(((x - 1) % 26) as usize)
          .unwrap(),
      );
      x = (x - 1) / 26;
    }
    name.chars().rev().collect()
  }

  pub(crate) fn n(self) -> u64 {
    self.0
  }

  pub(crate) fn population(self) -> u64 {
    let mut n = self.0;
    let mut population = 0;
    while n > 0 {
      population += n & 1;
      n >>= 1;
    }
    population
  }
}

impl PartialEq<u64> for Ordinal {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl Add<u64> for Ordinal {
  type Output = Self;

  fn add(self, other: u64) -> Ordinal {
    Ordinal::new(self.0 + other)
  }
}

impl AddAssign<u64> for Ordinal {
  fn add_assign(&mut self, other: u64) {
    *self = Ordinal::new(self.0 + other);
  }
}

impl FromStr for Ordinal {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let inner = s.parse()?;

    if inner >= Self::SUPPLY {
      return Err(format!("{} is not a valid ordinal", inner).into());
    }

    Ok(Self::new(inner))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn height() {
    assert_eq!(Ordinal::new(0).height(), 0);
    assert_eq!(Ordinal::new(1).height(), 0);
    assert_eq!(Ordinal::new(Epoch(0).subsidy()).height(), 1);
    assert_eq!(Ordinal::new(Epoch(0).subsidy() * 2).height(), 2);
    assert_eq!(
      Epoch(2).starting_ordinal().unwrap().height(),
      Epoch::BLOCKS * 2
    );
  }

  #[test]
  fn name() {
    assert_eq!(Ordinal::new(0).name(), "nvtdijuwxlo");
    assert_eq!(Ordinal::new(1).name(), "nvtdijuwxln");
    assert_eq!(Ordinal::new(26).name(), "nvtdijuwxko");
    assert_eq!(Ordinal::new(27).name(), "nvtdijuwxkn");
    assert_eq!(Ordinal::new(2099999997689999).name(), "");
  }

  #[test]
  fn population() {
    assert_eq!(Ordinal::new(0).population(), 0);
    assert_eq!(Ordinal::new(1).population(), 1);
    assert_eq!(
      Ordinal::new(0b11111111111111111111111111111111111111111111111111).population(),
      50
    );
  }

  #[test]
  fn epoch() {
    assert_eq!(Ordinal::new(0).epoch(), 0);
    assert_eq!(Ordinal::new(1).epoch(), 0);
    assert_eq!(Ordinal::new(1050000000000000).epoch(), 1);
  }

  #[test]
  fn block_position() {
    assert_eq!(Ordinal::new(0).subsidy_position(), 0);
    assert_eq!(Ordinal::new(1).subsidy_position(), 1);
    assert_eq!(
      Ordinal::new(Height(0).subsidy() - 1).subsidy_position(),
      Height(0).subsidy() - 1
    );
    assert_eq!(Ordinal::new(Height(0).subsidy()).subsidy_position(), 0);
    assert_eq!(Ordinal::new(Height(0).subsidy() + 1).subsidy_position(), 1);
    assert_eq!(
      Ordinal::new(Epoch(1).starting_ordinal().unwrap().n() + Epoch(1).subsidy())
        .subsidy_position(),
      0
    );
    assert_eq!(Ordinal::LAST.subsidy_position(), 0);
  }

  #[test]
  fn supply() {
    let mut mined = 0;

    for height in 0.. {
      let subsidy = Height(height).subsidy();

      if subsidy == 0 {
        break;
      }

      mined += subsidy;
    }

    assert_eq!(Ordinal::SUPPLY, mined);
  }
}
