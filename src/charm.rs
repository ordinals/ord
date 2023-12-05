#[derive(Copy, Clone)]
pub(crate) enum Charm {
  Coin,
  Cursed,
  Epic,
  Legendary,
  Lost,
  Nineball,
  Rare,
  Reinscription,
  Unbound,
  Uncommon,
}

impl Charm {
  pub(crate) const ALL: [Charm; 10] = [
    Self::Coin,
    Self::Uncommon,
    Self::Rare,
    Self::Epic,
    Self::Legendary,
    Self::Nineball,
    Self::Reinscription,
    Self::Cursed,
    Self::Unbound,
    Self::Lost,
  ];

  fn flag(self) -> u16 {
    1 << self as u16
  }

  pub(crate) fn set(self, charms: &mut u16) {
    *charms |= self.flag();
  }

  pub(crate) fn is_set(self, charms: u16) -> bool {
    charms & self.flag() != 0
  }

  pub(crate) fn icon(self) -> &'static str {
    match self {
      Self::Coin => "🪙",
      Self::Cursed => "👹",
      Self::Epic => "🪻",
      Self::Legendary => "🌝",
      Self::Lost => "🤔",
      Self::Nineball => "9️⃣",
      Self::Rare => "🧿",
      Self::Reinscription => "♻️",
      Self::Unbound => "🔓",
      Self::Uncommon => "🌱",
    }
  }

  pub(crate) fn title(self) -> &'static str {
    match self {
      Self::Coin => "coin",
      Self::Cursed => "cursed",
      Self::Epic => "epic",
      Self::Legendary => "legendary",
      Self::Lost => "lost",
      Self::Nineball => "nineball",
      Self::Rare => "rare",
      Self::Reinscription => "reinscription",
      Self::Unbound => "unbound",
      Self::Uncommon => "uncommon",
    }
  }
}
