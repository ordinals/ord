use super::*;

#[derive(Copy, Clone)]
pub(crate) struct Ordinal(u64);

impl Ordinal {
  pub(crate) fn new(inner: u64) -> Result<Self> {
    if inner > 2099999997689999 {
      return Err(
        format!(
          "{} is greater than 2099999997689999, the last ordinal",
          inner
        )
        .into(),
      );
    }

    Ok(Self(inner))
  }

  pub(crate) fn height(self) -> u64 {
    // TODO: Fix
    self.0 / (50 * 100_000_000)
  }

  pub(crate) fn position_in_coinbase(self) -> u64 {
    // TODO: Fix
    self.0 % (50 * 100_000_000)
  }

  pub(crate) fn number(self) -> u64 {
    self.0
  }
}

impl FromStr for Ordinal {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s.parse()?)
  }
}
