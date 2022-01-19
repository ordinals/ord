use super::*;

pub(crate) const SUPPLY: u64 = 2099999997690000;

pub(crate) const INITIAL_SUBSIDY: u64 = 50 * COIN_VALUE;

pub(crate) const EPOCHS: u64 = EPOCH_ORDINALS.len() as u64;

pub(crate) const EPOCH_BLOCKS: u64 = 210000;

pub(crate) const EPOCH_ORDINALS: &[u64] = &[
  0,
  1050000000000000,
  1575000000000000,
  1837500000000000,
  1968750000000000,
  2034375000000000,
  2067187500000000,
  2083593750000000,
  2091796875000000,
  2095898437500000,
  2097949218750000,
  2098974609270000,
  2099487304530000,
  2099743652160000,
  2099871825870000,
  2099935912620000,
  2099967955890000,
  2099983977420000,
  2099991988080000,
  2099995993410000,
  2099997995970000,
  2099998997250000,
  2099999497890000,
  2099999748210000,
  2099999873370000,
  2099999935950000,
  2099999967240000,
  2099999982780000,
  2099999990550000,
  2099999994330000,
  2099999996220000,
  2099999997060000,
  2099999997480000,
];

#[cfg(test)]
mod tests {
  use super::super::*;

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

    assert_eq!(SUPPLY, mined);
  }

  #[test]
  fn epochs() {
    assert!(Height(EPOCHS * 210_000).subsidy() == 0);
    assert!(Height(EPOCHS * 210_000 - 1).subsidy() == 1);
    assert_eq!(EPOCHS, 33);
  }

  #[test]
  fn epoch_ordinals() {
    let mut ordinal = 0;

    let mut epoch_ordinals = Vec::new();

    for epoch in 0..33 {
      epoch_ordinals.push(ordinal);
      ordinal += EPOCH_BLOCKS * Height(epoch * EPOCH_BLOCKS).subsidy();
    }

    assert_eq!(EPOCH_ORDINALS, epoch_ordinals);

    assert_eq!(EPOCH_ORDINALS.len(), 33);
  }
}
