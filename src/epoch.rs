use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
pub(crate) struct Epoch(pub(crate) u64);

impl Epoch {
  pub(crate) const STARTING_ORDINALS: &'static [Ordinal] = &[
    Ordinal(0),
    Ordinal(1050000000000000),
    Ordinal(1575000000000000),
    Ordinal(1837500000000000),
    Ordinal(1968750000000000),
    Ordinal(2034375000000000),
    Ordinal(2067187500000000),
    Ordinal(2083593750000000),
    Ordinal(2091796875000000),
    Ordinal(2095898437500000),
    Ordinal(2097949218750000),
    Ordinal(2098974609270000),
    Ordinal(2099487304530000),
    Ordinal(2099743652160000),
    Ordinal(2099871825870000),
    Ordinal(2099935912620000),
    Ordinal(2099967955890000),
    Ordinal(2099983977420000),
    Ordinal(2099991988080000),
    Ordinal(2099995993410000),
    Ordinal(2099997995970000),
    Ordinal(2099998997250000),
    Ordinal(2099999497890000),
    Ordinal(2099999748210000),
    Ordinal(2099999873370000),
    Ordinal(2099999935950000),
    Ordinal(2099999967240000),
    Ordinal(2099999982780000),
    Ordinal(2099999990550000),
    Ordinal(2099999994330000),
    Ordinal(2099999996220000),
    Ordinal(2099999997060000),
    Ordinal(2099999997480000),
    Ordinal(Ordinal::SUPPLY),
  ];

  pub(crate) const BLOCKS: u64 = 210000;

  pub(crate) fn subsidy(self) -> u64 {
    if self.0 < 64 {
      (50 * COIN_VALUE) >> self.0
    } else {
      0
    }
  }

  pub(crate) fn starting_ordinal(self) -> Ordinal {
    *Self::STARTING_ORDINALS
      .get(self.0 as usize)
      .unwrap_or_else(|| Self::STARTING_ORDINALS.last().unwrap())
  }

  pub(crate) fn starting_height(self) -> Height {
    Height(self.0 * Self::BLOCKS)
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

impl From<Height> for Epoch {
  fn from(height: Height) -> Self {
    Self(height.0 / Self::BLOCKS)
  }
}

#[cfg(test)]
mod tests {
  use super::super::*;

  #[test]
  fn starting_ordinal() {
    assert_eq!(Epoch(0).starting_ordinal(), 0);
    assert_eq!(
      Epoch(1).starting_ordinal(),
      Epoch(0).subsidy() * Epoch::BLOCKS
    );
    assert_eq!(
      Epoch(2).starting_ordinal(),
      (Epoch(0).subsidy() + Epoch(1).subsidy()) * Epoch::BLOCKS
    );
    assert_eq!(Epoch(33).starting_ordinal(), Ordinal(Ordinal::SUPPLY));
    assert_eq!(Epoch(34).starting_ordinal(), Ordinal(Ordinal::SUPPLY));
  }

  #[test]
  fn starting_ordinals() {
    let mut ordinal = 0;

    let mut epoch_ordinals = Vec::new();

    for epoch in 0..34 {
      epoch_ordinals.push(ordinal);
      ordinal += Epoch::BLOCKS * Epoch(epoch).subsidy();
    }

    assert_eq!(Epoch::STARTING_ORDINALS, epoch_ordinals);
    assert_eq!(Epoch::STARTING_ORDINALS.len(), 34);
  }

  #[test]
  fn subsidy() {
    assert_eq!(Epoch(0).subsidy(), 5000000000);
    assert_eq!(Epoch(1).subsidy(), 2500000000);
    assert_eq!(Epoch(32).subsidy(), 1);
    assert_eq!(Epoch(33).subsidy(), 0);
  }

  #[test]
  fn blocks() {
    // c.f. https://github.com/bitcoin/bitcoin/blob/master/src/chainparams.cpp
    assert_eq!(Epoch::BLOCKS, 210000);
  }

  #[test]
  fn starting_height() {
    assert_eq!(Epoch(0).starting_height(), 0);
    assert_eq!(Epoch(1).starting_height(), Epoch::BLOCKS);
    assert_eq!(Epoch(2).starting_height(), Epoch::BLOCKS * 2);
  }

  #[test]
  fn from_height() {
    assert_eq!(Epoch::from(Height(0)), 0);
    assert_eq!(Epoch::from(Height(Epoch::BLOCKS)), 1);
    assert_eq!(Epoch::from(Height(Epoch::BLOCKS) + 1), 1);
  }

  #[test]
  fn from_ordinal() {
    assert_eq!(Epoch::from(Ordinal(0)), 0);
    assert_eq!(Epoch::from(Ordinal(1)), 0);
    assert_eq!(Epoch::from(Epoch(1).starting_ordinal()), 1);
    assert_eq!(Epoch::from(Epoch(1).starting_ordinal() + 1), 1);
  }

  #[test]
  fn eq() {
    assert_eq!(Epoch(0), 0);
    assert_eq!(Epoch(100), 100);
  }
}
