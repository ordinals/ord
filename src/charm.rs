#[derive(Copy, Clone)]
pub(crate) enum Charm {
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
  pub(crate) const ALL: [Charm; 9] = [
    Charm::Uncommon,
    Charm::Rare,
    Charm::Epic,
    Charm::Legendary,
    Charm::Nineball,
    Charm::Reinscription,
    Charm::Cursed,
    Charm::Unbound,
    Charm::Lost,
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
      Charm::Cursed => "👹",
      Charm::Epic => "🪻",
      Charm::Legendary => "🌝",
      Charm::Lost => "🤔",
      Charm::Nineball => "9️⃣",
      Charm::Rare => "🧿",
      Charm::Reinscription => "♻️",
      Charm::Unbound => "🔓",
      Charm::Uncommon => "🌱",
    }
  }

  pub(crate) fn title(self) -> &'static str {
    match self {
      Charm::Cursed => "cursed",
      Charm::Epic => "epic",
      Charm::Legendary => "legendary",
      Charm::Lost => "lost",
      Charm::Nineball => "nineball",
      Charm::Rare => "rare",
      Charm::Reinscription => "reinscription",
      Charm::Unbound => "unbound",
      Charm::Uncommon => "uncommon",
    }
  }
}
