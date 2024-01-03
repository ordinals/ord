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
