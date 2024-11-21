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
  Mythic = 11,
  Burned = 12,
  Palindrome = 13,
}

impl Charm {
  pub const ALL: [Self; 14] = [
    Self::Coin,
    Self::Uncommon,
    Self::Rare,
    Self::Epic,
    Self::Legendary,
    Self::Mythic,
    Self::Nineball,
    Self::Palindrome,
    Self::Reinscription,
    Self::Cursed,
    Self::Unbound,
    Self::Lost,
    Self::Vindicated,
    Self::Burned,
  ];

  pub fn flag(self) -> u16 {
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
      Self::Burned => "🔥",
      Self::Coin => "🪙",
      Self::Cursed => "👹",
      Self::Epic => "🪻",
      Self::Legendary => "🌝",
      Self::Lost => "🤔",
      Self::Mythic => "🎃",
      Self::Nineball => "\u{39}\u{fe0f}\u{20e3}",
      Self::Palindrome => "🦋",
      Self::Rare => "🧿",
      Self::Reinscription => "♻️",
      Self::Unbound => "🔓",
      Self::Uncommon => "🌱",
      Self::Vindicated => "\u{2764}\u{fe0f}\u{200d}\u{1f525}",
    }
  }

  pub fn charms(charms: u16) -> Vec<Charm> {
    Self::ALL
      .into_iter()
      .filter(|charm| charm.is_set(charms))
      .collect()
  }
}

impl Display for Charm {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Burned => "burned",
        Self::Coin => "coin",
        Self::Cursed => "cursed",
        Self::Epic => "epic",
        Self::Legendary => "legendary",
        Self::Lost => "lost",
        Self::Mythic => "mythic",
        Self::Nineball => "nineball",
        Self::Palindrome => "palindrome",
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
      "burned" => Self::Burned,
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
