use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Charms(pub u16);

impl Charms {
  pub fn new() -> Self {
    Self(0)
  }

  pub fn set(&mut self, charm: Charm) -> Self {
    self.0 |= 1 << charm as u16;

    self.clone()
  }

  pub fn unset(&self, charm: Charm) -> Self {
    Self(self.0 & !(1 << charm as u16))
  }

  pub fn union(&mut self, other: Self) -> Self {
    self.0 |= other.0;

    self.clone()
  }

  pub fn is_set(&self, charm: Charm) -> bool {
    self.0 & (1 << charm as u16) != 0
  }

  pub fn is_empty(&self) -> bool {
    self.0 == 0
  }

  pub fn active_charms(&self) -> Vec<Charm> {
    Charm::ALL
      .iter()
      .copied()
      .filter(|&c| self.is_set(c))
      .collect()
  }
}

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
  Mythic = 11,
}

impl Charm {
  pub const ALL: [Charm; 12] = [
    Self::Coin,
    Self::Uncommon,
    Self::Rare,
    Self::Epic,
    Self::Legendary,
    Self::Mythic,
    Self::Nineball,
    Self::Reinscription,
    Self::Cursed,
    Self::Unbound,
    Self::Lost,
    Self::Vindicated,
  ];

  pub fn icon(self) -> &'static str {
    match self {
      Self::Coin => "🪙",
      Self::Cursed => "👹",
      Self::Epic => "🪻",
      Self::Legendary => "🌝",
      Self::Lost => "🤔",
      Self::Mythic => "🎃",
      Self::Nineball => "9️⃣",
      Self::Rare => "🧿",
      Self::Reinscription => "♻️",
      Self::Unbound => "🔓",
      Self::Uncommon => "🌱",
      Self::Vindicated => "❤️‍🔥",
    }
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
        Self::Mythic => "mythic",
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
      "mythic" => Self::Mythic,
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
  fn set() {
    let mut charms = Charms::new();
    assert!(!charms.is_set(Charm::Coin));
    charms.set(Charm::Coin);
    assert!(charms.is_set(Charm::Coin));
  }

  #[test]
  fn unset() {
    let mut charms = Charms::new();
    charms.set(Charm::Coin);
    assert!(charms.is_set(Charm::Coin));
    let charms = charms.unset(Charm::Coin);
    assert!(!charms.is_set(Charm::Coin));
  }
}
