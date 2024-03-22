use super::*;

#[derive(Copy, Clone, Debug, PartialEq, DeserializeFromStr, SerializeDisplay)]
pub enum Charm {
  Coin = 0,
  Cursed = 1,
  Epic = 2,
  Legendary = 3,
  Lost = 4,
  Nineball = 5,
  Rare = 6,
  Reinscription = 7,
  Unbound = 8,
  Uncommon = 9,
  Vindicated = 10,
}

impl Charm {
  pub const ALL: [Charm; 11] = [
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
    Self::Vindicated,
  ];

  fn flag(self) -> u16 {
    1 << self as u16
  }

  pub fn set(self, charms: &mut u16) {
    *charms |= self.flag();
  }

  pub fn is_set(self, charms: u16) -> bool {
    charms & self.flag() != 0
  }

  pub fn unset(self, charms: u16) -> u16 {
    charms & !self.flag()
  }

  pub fn icon(self) -> &'static str {
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
      Self::Vindicated => "❤️‍🔥",
    }
  }

  pub fn charms(charms: u16) -> Vec<Charm> {
    Self::ALL
      .iter()
      .filter(|charm| charm.is_set(charms))
      .copied()
      .collect()
  }
}

impl Display for Charm {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
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
        Self::Vindicated => "vindicated",
      }
    )
  }
}

impl FromStr for Charm {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "coin" => Self::Coin,
      "cursed" => Self::Cursed,
      "epic" => Self::Epic,
      "legendary" => Self::Legendary,
      "lost" => Self::Lost,
      "nineball" => Self::Nineball,
      "rare" => Self::Rare,
      "reinscription" => Self::Reinscription,
      "unbound" => Self::Unbound,
      "uncommon" => Self::Uncommon,
      "vindicated" => Self::Vindicated,
      _ => return Err(format!("invalid charm `{s}`")),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn flag() {
    assert_eq!(Charm::Coin.flag(), 0b1);
    assert_eq!(Charm::Cursed.flag(), 0b10);
  }

  #[test]
  fn set() {
    let mut flags = 0;
    assert!(!Charm::Coin.is_set(flags));
    Charm::Coin.set(&mut flags);
    assert!(Charm::Coin.is_set(flags));
  }

  #[test]
  fn unset() {
    let mut flags = 0;
    Charm::Coin.set(&mut flags);
    assert!(Charm::Coin.is_set(flags));
    let flags = Charm::Coin.unset(flags);
    assert!(!Charm::Coin.is_set(flags));
  }
}
