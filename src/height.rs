use super::*;

#[derive(Copy, Clone, Debug, Display)]
pub(crate) struct Height(pub(crate) u64);

impl Height {
  pub(crate) fn n(self) -> u64 {
    self.0
  }

  pub(crate) fn subsidy(self) -> u64 {
    let halvings = self.0 / Epoch::BLOCKS;

    if halvings < 64 {
      (50 * COIN_VALUE) >> halvings
    } else {
      0
    }
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
  fn subsidy() {
    assert_eq!(Height(0).subsidy(), 5000000000);
    assert_eq!(Height(1).subsidy(), 5000000000);
    assert_eq!(Height(210000 - 1).subsidy(), 5000000000);
    assert_eq!(Height(210000).subsidy(), 2500000000);
    assert_eq!(Height(210000 + 1).subsidy(), 2500000000);
  }

  // #[test]
  // fn mineds() {
  //   assert_eq!(mined(0), 0);
  //   assert_eq!(mined(1), 50 * COIN_VALUE);
  // }

  // #[test]
  // fn names() {
  //   assert_eq!(name(Ordinal::new(0)), "nvtdijuwxlo");
  //   assert_eq!(name(Ordinal::new(1)), "nvtdijuwxln");
  //   assert_eq!(name(Ordinal::new(26)), "nvtdijuwxko");
  //   assert_eq!(name(Ordinal::new(27)), "nvtdijuwxkn");
  //   assert_eq!(name(Ordinal::new(2099999997689999)), "");
  // }

  // #[test]
  // fn populations() {
  //   assert_eq!(population(0), 0);
  //   assert_eq!(population(1), 1);
  //   assert_eq!(population(u64::max_value()), 64);
  // }
}
