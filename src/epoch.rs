use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, PartialOrd)]
pub(crate) struct Epoch(pub(crate) u64);

impl Epoch {
  pub(crate) const STARTING_ORDINALS: [Ordinal; 34] = [
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
  pub(crate) const FIRST_POST_SUBSIDY: Epoch = Self(33);

  pub(crate) fn subsidy(self) -> u64 {
    if self < Self::FIRST_POST_SUBSIDY {
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
    Height(self.0 * SUBSIDY_HALVING_INTERVAL)
  }
}

impl PartialEq<u64> for Epoch {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl From<Ordinal> for Epoch {
  fn from(ordinal: Ordinal) -> Self {
    if ordinal < Self::STARTING_ORDINALS[1] {
      Epoch(0)
    } else if ordinal < Self::STARTING_ORDINALS[2] {
      Epoch(1)
    } else if ordinal < Self::STARTING_ORDINALS[3] {
      Epoch(2)
    } else if ordinal < Self::STARTING_ORDINALS[4] {
      Epoch(3)
    } else if ordinal < Self::STARTING_ORDINALS[5] {
      Epoch(4)
    } else if ordinal < Self::STARTING_ORDINALS[6] {
      Epoch(5)
    } else if ordinal < Self::STARTING_ORDINALS[7] {
      Epoch(6)
    } else if ordinal < Self::STARTING_ORDINALS[8] {
      Epoch(7)
    } else if ordinal < Self::STARTING_ORDINALS[9] {
      Epoch(8)
    } else if ordinal < Self::STARTING_ORDINALS[10] {
      Epoch(9)
    } else if ordinal < Self::STARTING_ORDINALS[11] {
      Epoch(10)
    } else if ordinal < Self::STARTING_ORDINALS[12] {
      Epoch(11)
    } else if ordinal < Self::STARTING_ORDINALS[13] {
      Epoch(12)
    } else if ordinal < Self::STARTING_ORDINALS[14] {
      Epoch(13)
    } else if ordinal < Self::STARTING_ORDINALS[15] {
      Epoch(14)
    } else if ordinal < Self::STARTING_ORDINALS[16] {
      Epoch(15)
    } else if ordinal < Self::STARTING_ORDINALS[17] {
      Epoch(16)
    } else if ordinal < Self::STARTING_ORDINALS[18] {
      Epoch(17)
    } else if ordinal < Self::STARTING_ORDINALS[19] {
      Epoch(18)
    } else if ordinal < Self::STARTING_ORDINALS[20] {
      Epoch(19)
    } else if ordinal < Self::STARTING_ORDINALS[21] {
      Epoch(20)
    } else if ordinal < Self::STARTING_ORDINALS[22] {
      Epoch(21)
    } else if ordinal < Self::STARTING_ORDINALS[23] {
      Epoch(22)
    } else if ordinal < Self::STARTING_ORDINALS[24] {
      Epoch(23)
    } else if ordinal < Self::STARTING_ORDINALS[25] {
      Epoch(24)
    } else if ordinal < Self::STARTING_ORDINALS[26] {
      Epoch(25)
    } else if ordinal < Self::STARTING_ORDINALS[27] {
      Epoch(26)
    } else if ordinal < Self::STARTING_ORDINALS[28] {
      Epoch(27)
    } else if ordinal < Self::STARTING_ORDINALS[29] {
      Epoch(28)
    } else if ordinal < Self::STARTING_ORDINALS[30] {
      Epoch(29)
    } else if ordinal < Self::STARTING_ORDINALS[31] {
      Epoch(30)
    } else if ordinal < Self::STARTING_ORDINALS[32] {
      Epoch(31)
    } else if ordinal < Self::STARTING_ORDINALS[33] {
      Epoch(32)
    } else {
      Epoch(33)
    }
  }
}

impl From<Height> for Epoch {
  fn from(height: Height) -> Self {
    Self(height.0 / SUBSIDY_HALVING_INTERVAL)
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
      Epoch(0).subsidy() * SUBSIDY_HALVING_INTERVAL
    );
    assert_eq!(
      Epoch(2).starting_ordinal(),
      (Epoch(0).subsidy() + Epoch(1).subsidy()) * SUBSIDY_HALVING_INTERVAL
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
      ordinal += SUBSIDY_HALVING_INTERVAL * Epoch(epoch).subsidy();
    }

    assert_eq!(Epoch::STARTING_ORDINALS.as_slice(), epoch_ordinals);
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
  fn starting_height() {
    assert_eq!(Epoch(0).starting_height(), 0);
    assert_eq!(Epoch(1).starting_height(), SUBSIDY_HALVING_INTERVAL);
    assert_eq!(Epoch(2).starting_height(), SUBSIDY_HALVING_INTERVAL * 2);
  }

  #[test]
  fn from_height() {
    assert_eq!(Epoch::from(Height(0)), 0);
    assert_eq!(Epoch::from(Height(SUBSIDY_HALVING_INTERVAL)), 1);
    assert_eq!(Epoch::from(Height(SUBSIDY_HALVING_INTERVAL) + 1), 1);
  }

  #[test]
  fn from_ordinal() {
    for (epoch, starting_ordinal) in Epoch::STARTING_ORDINALS.into_iter().enumerate() {
      if epoch > 0 {
        assert_eq!(
          Epoch::from(Ordinal(starting_ordinal.n() - 1)),
          Epoch(epoch as u64 - 1)
        );
      }
      assert_eq!(Epoch::from(starting_ordinal), Epoch(epoch as u64));
      assert_eq!(Epoch::from(starting_ordinal + 1), Epoch(epoch as u64));
    }
    assert_eq!(Epoch::from(Ordinal(0)), 0);
    assert_eq!(Epoch::from(Ordinal(1)), 0);
    assert_eq!(Epoch::from(Epoch(1).starting_ordinal()), 1);
    assert_eq!(Epoch::from(Epoch(1).starting_ordinal() + 1), 1);
    assert_eq!(Epoch::from(Ordinal(u64::max_value())), 33);
  }

  #[test]
  fn eq() {
    assert_eq!(Epoch(0), 0);
    assert_eq!(Epoch(100), 100);
  }

  #[test]
  fn first_post_subsidy() {
    assert_eq!(Epoch::FIRST_POST_SUBSIDY.subsidy(), 0);
    assert!((Epoch(Epoch::FIRST_POST_SUBSIDY.0 - 1)).subsidy() > 0);
  }
}
