use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct Epoch(u64);

impl Epoch {
  const STARTING_ORDINALS: &'static [Ordinal] = &[
    Ordinal::new(0),
    Ordinal::new(1050000000000000),
    Ordinal::new(1575000000000000),
    Ordinal::new(1837500000000000),
    Ordinal::new(1968750000000000),
    Ordinal::new(2034375000000000),
    Ordinal::new(2067187500000000),
    Ordinal::new(2083593750000000),
    Ordinal::new(2091796875000000),
    Ordinal::new(2095898437500000),
    Ordinal::new(2097949218750000),
    Ordinal::new(2098974609270000),
    Ordinal::new(2099487304530000),
    Ordinal::new(2099743652160000),
    Ordinal::new(2099871825870000),
    Ordinal::new(2099935912620000),
    Ordinal::new(2099967955890000),
    Ordinal::new(2099983977420000),
    Ordinal::new(2099991988080000),
    Ordinal::new(2099995993410000),
    Ordinal::new(2099997995970000),
    Ordinal::new(2099998997250000),
    Ordinal::new(2099999497890000),
    Ordinal::new(2099999748210000),
    Ordinal::new(2099999873370000),
    Ordinal::new(2099999935950000),
    Ordinal::new(2099999967240000),
    Ordinal::new(2099999982780000),
    Ordinal::new(2099999990550000),
    Ordinal::new(2099999994330000),
    Ordinal::new(2099999996220000),
    Ordinal::new(2099999997060000),
    Ordinal::new(2099999997480000),
  ];

  const LAST: Epoch = Self(Self::STARTING_ORDINALS.len() as u64 - 1);

  pub(crate) const BLOCKS: u64 = 210000;

  pub(crate) const fn new(inner: u64) -> Self {
    assert!(inner <= Self::LAST.0);
    Self(inner)
  }

  pub(crate) fn n(self) -> u64 {
    self.0
  }

  pub(crate) fn subsidy(self) -> u64 {
    Height(self.0 * Self::BLOCKS).subsidy()
  }

  pub(crate) fn starting_ordinal(self) -> Ordinal {
    Self::STARTING_ORDINALS[self.0 as usize]
  }

  pub(crate) fn starting_height(self) -> u64 {
    self.0 * Self::BLOCKS
  }
}

impl PartialEq<u64> for Epoch {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl From<Ordinal> for Epoch {
  fn from(ordinal: Ordinal) -> Self {
    match Self::STARTING_ORDINALS.binary_search(&ordinal) {
      Ok(i) => Epoch(i as u64),
      Err(i) => Epoch(i as u64 - 1),
    }
  }
}
