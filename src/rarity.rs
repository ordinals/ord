use super::*;

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum Rarity {
  Common,
  Uncommon,
  Rare,
  Epic,
  Legendary,
  Mythic,
}

impl Display for Rarity {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Common => "common",
        Self::Uncommon => "uncommon",
        Self::Rare => "rare",
        Self::Epic => "epic",
        Self::Legendary => "legendary",
        Self::Mythic => "mythic",
      }
    )
  }
}

impl From<Ordinal> for Rarity {
  fn from(ordinal: Ordinal) -> Self {
    let Degree {
      hour,
      minute,
      second,
      third,
    } = ordinal.degree();

    if hour == 0 && minute == 0 && second == 0 && third == 0 {
      Self::Mythic
    } else if minute == 0 && second == 0 && third == 0 {
      Self::Legendary
    } else if minute == 0 && third == 0 {
      Self::Epic
    } else if second == 0 && third == 0 {
      Self::Rare
    } else if third == 0 {
      Self::Uncommon
    } else {
      Self::Common
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rarity() {
    assert_eq!(Ordinal(0).rarity(), Rarity::Mythic);
    assert_eq!(Ordinal(1).rarity(), Rarity::Common);

    assert_eq!(Ordinal(50 * 100_000_000 - 1).rarity(), Rarity::Common);
    assert_eq!(Ordinal(50 * 100_000_000).rarity(), Rarity::Uncommon);
    assert_eq!(Ordinal(50 * 100_000_000 + 1).rarity(), Rarity::Common);

    assert_eq!(
      Ordinal(50 * 100_000_000 * 2016 - 1).rarity(),
      Rarity::Common
    );
    assert_eq!(Ordinal(50 * 100_000_000 * 2016).rarity(), Rarity::Rare);
    assert_eq!(
      Ordinal(50 * 100_000_000 * 2016 + 1).rarity(),
      Rarity::Common
    );

    assert_eq!(
      Ordinal(50 * 100_000_000 * 210000 - 1).rarity(),
      Rarity::Common
    );
    assert_eq!(Ordinal(50 * 100_000_000 * 210000).rarity(), Rarity::Epic);
    assert_eq!(
      Ordinal(50 * 100_000_000 * 210000 + 1).rarity(),
      Rarity::Common
    );

    assert_eq!(Ordinal(2067187500000000 - 1).rarity(), Rarity::Common);
    assert_eq!(Ordinal(2067187500000000).rarity(), Rarity::Legendary);
    assert_eq!(Ordinal(2067187500000000 + 1).rarity(), Rarity::Common);
  }
}
