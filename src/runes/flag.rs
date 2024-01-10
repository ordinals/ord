pub(super) enum Flag {
  Etch = 0,
  #[allow(unused)]
  Burn = 127,
}

impl Flag {
  pub(super) fn mask(self) -> u128 {
    1 << self as u128
  }

  pub(super) fn take(self, flags: &mut u128) -> bool {
    let mask = self.mask();
    let set = *flags & mask != 0;
    *flags &= !mask;
    set
  }

  pub(super) fn set(self, flags: &mut u128) {
    *flags |= self.mask()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mask() {
    assert_eq!(Flag::Etch.mask(), 0b1);
    assert_eq!(Flag::Burn.mask(), 1 << 127);
  }

  #[test]
  fn take() {
    let mut flags = 1;
    assert!(Flag::Etch.take(&mut flags));
    assert_eq!(flags, 0);

    let mut flags = 0;
    assert!(!Flag::Etch.take(&mut flags));
    assert_eq!(flags, 0);
  }

  #[test]
  fn set() {
    let mut flags = 0;
    Flag::Etch.set(&mut flags);
    assert_eq!(flags, 1);
  }
}
