use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct Ordinal(pub(crate) u64);

impl Ordinal {
  pub(crate) const LAST: Self = Self(Self::SUPPLY - 1);
  pub(crate) const SUPPLY: u64 = 2099999997690000;

  pub(crate) fn n(self) -> u64 {
    self.0
  }

  pub(crate) fn degree(self) -> Degree {
    self.into()
  }

  pub(crate) fn height(self) -> Height {
    self.epoch().starting_height() + self.epoch_position() / self.epoch().subsidy()
  }

  pub(crate) fn cycle(self) -> u64 {
    Epoch::from(self).0 / CYCLE_EPOCHS
  }

  pub(crate) fn epoch(self) -> Epoch {
    self.into()
  }

  pub(crate) fn period(self) -> u64 {
    self.height().n() / PERIOD_BLOCKS
  }

  pub(crate) fn third(self) -> u64 {
    self.epoch_position() % self.epoch().subsidy()
  }

  pub(crate) fn epoch_position(self) -> u64 {
    self.0 - self.epoch().starting_ordinal().0
  }

  pub(crate) fn decimal(self) -> String {
    format!("{}.{}", self.height(), self.third())
  }

  pub(crate) fn rarity(self) -> &'static str {
    let Degree {
      hour,
      minute,
      second,
      third,
    } = self.degree();

    if hour == 0 && minute == 0 && second == 0 && third == 0 {
      "mythic"
    } else if minute == 0 && second == 0 && third == 0 {
      "legendary"
    } else if minute == 0 && third == 0 {
      "epic"
    } else if second == 0 && third == 0 {
      "rare"
    } else if third == 0 {
      "uncommon"
    } else {
      "common"
    }
  }

  pub(crate) fn name(self) -> String {
    let mut x = Self::SUPPLY - self.0;
    let mut name = String::new();
    while x > 0 {
      name.push(
        "abcdefghijklmnopqrstuvwxyz"
          .chars()
          .nth(((x - 1) % 26) as usize)
          .unwrap(),
      );
      x = (x - 1) / 26;
    }
    name.chars().rev().collect()
  }

  fn from_name(s: &str) -> Result<Self> {
    let mut x = 0;
    for c in s.chars() {
      match c {
        'a'..='z' => {
          x = x * 26 + c as u64 - 'a' as u64 + 1;
        }
        _ => bail!("Invalid character in ordinal name: {c}"),
      }
    }
    if x > Self::SUPPLY {
      bail!("Ordinal name out of range");
    }
    Ok(Ordinal(Self::SUPPLY - x))
  }

  fn from_degree(s: &str) -> Result<Self> {
    let (cycle_number, rest) = s
      .split_once('°')
      .ok_or_else(|| anyhow!("Missing degree symbol"))?;
    let cycle_number = cycle_number.parse::<u64>()?;

    let (epoch_offset, rest) = rest
      .split_once('′')
      .ok_or_else(|| anyhow!("Missing minute symbol"))?;
    let epoch_offset = epoch_offset.parse::<u64>()?;
    if epoch_offset >= Epoch::BLOCKS {
      bail!("Invalid epoch offset");
    }

    let (period_offset, rest) = rest
      .split_once('″')
      .ok_or_else(|| anyhow!("Missing second symbol"))?;
    let period_offset = period_offset.parse::<u64>()?;
    if period_offset >= PERIOD_BLOCKS {
      bail!("Invalid period offset");
    }

    let cycle_start_epoch = cycle_number * CYCLE_EPOCHS;

    let cycle_progression = period_offset
      .checked_sub(epoch_offset % PERIOD_BLOCKS)
      .ok_or_else(|| anyhow!("Invalid relationship between epoch offset and period offset"))?;

    if cycle_progression % (Epoch::BLOCKS % PERIOD_BLOCKS) != 0 {
      bail!("Invalid relationship between epoch offset and period offset");
    }

    let epochs_since_cycle_start = cycle_progression / (Epoch::BLOCKS % PERIOD_BLOCKS);

    let epoch = cycle_start_epoch + epochs_since_cycle_start;

    let height = Height(epoch * Epoch::BLOCKS + epoch_offset);

    let (block_offset, rest) = match rest.split_once('‴') {
      Some((block_offset, rest)) => (block_offset.parse::<u64>()?, rest),
      None => (0, rest),
    };

    if !rest.is_empty() {
      bail!("Trailing characters");
    }

    if block_offset >= height.subsidy() {
      bail!("Invalid block offset");
    }

    Ok(height.starting_ordinal() + block_offset)
  }

  fn from_decimal(s: &str) -> Result<Self> {
    let (height, offset) = s.split_once('.').ok_or_else(|| anyhow!("Missing period"))?;
    let height = Height(height.parse()?);
    let offset = offset.parse::<u64>()?;

    if offset >= height.subsidy() {
      bail!("Invalid block offset");
    }

    Ok(height.starting_ordinal() + offset)
  }
}

impl PartialEq<u64> for Ordinal {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl Add<u64> for Ordinal {
  type Output = Self;

  fn add(self, other: u64) -> Ordinal {
    Ordinal(self.0 + other)
  }
}

impl AddAssign<u64> for Ordinal {
  fn add_assign(&mut self, other: u64) {
    *self = Ordinal(self.0 + other);
  }
}

impl FromStr for Ordinal {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    if s.chars().any(|c| matches!(c, 'a'..='z')) {
      Self::from_name(s)
    } else if s.contains('°') {
      Self::from_degree(s)
    } else if s.contains('.') {
      Self::from_decimal(s)
    } else {
      let ordinal = Self(s.parse()?);
      if ordinal > Self::LAST {
        Err(anyhow!("Invalid ordinal"))
      } else {
        Ok(ordinal)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn n() {
    assert_eq!(Ordinal(1).n(), 1);
    assert_eq!(Ordinal(100).n(), 100);
  }

  #[test]
  fn height() {
    assert_eq!(Ordinal(0).height(), 0);
    assert_eq!(Ordinal(1).height(), 0);
    assert_eq!(Ordinal(Epoch(0).subsidy()).height(), 1);
    assert_eq!(Ordinal(Epoch(0).subsidy() * 2).height(), 2);
    assert_eq!(Epoch(2).starting_ordinal().height(), Epoch::BLOCKS * 2);
    assert_eq!(Ordinal(50 * 100_000_000).height(), 1);
    assert_eq!(Ordinal(2099999997689999).height(), 6929999);
    assert_eq!(Ordinal(2099999997689998).height(), 6929998);
  }

  #[test]
  fn name() {
    assert_eq!(Ordinal(0).name(), "nvtdijuwxlp");
    assert_eq!(Ordinal(1).name(), "nvtdijuwxlo");
    assert_eq!(Ordinal(26).name(), "nvtdijuwxkp");
    assert_eq!(Ordinal(27).name(), "nvtdijuwxko");
    assert_eq!(Ordinal(2099999997689999).name(), "a");
    assert_eq!(Ordinal(2099999997689999 - 1).name(), "b");
    assert_eq!(Ordinal(2099999997689999 - 25).name(), "z");
    assert_eq!(Ordinal(2099999997689999 - 26).name(), "aa");
  }

  #[test]
  fn number() {
    assert_eq!(Ordinal(2099999997689999).n(), 2099999997689999);
  }

  #[test]
  fn decimal() {
    assert_eq!(Ordinal(2099999997689999).decimal(), "6929999.0");
  }

  #[test]
  fn degree() {
    assert_eq!(Ordinal(0).degree().to_string(), "0°0′0″0‴");
    assert_eq!(Ordinal(1).degree().to_string(), "0°0′0″1‴");
    assert_eq!(
      Ordinal(50 * 100_000_000 - 1).degree().to_string(),
      "0°0′0″4999999999‴"
    );
    assert_eq!(Ordinal(50 * 100_000_000).degree().to_string(), "0°1′1″0‴");
    assert_eq!(
      Ordinal(50 * 100_000_000 + 1).degree().to_string(),
      "0°1′1″1‴"
    );
    assert_eq!(
      Ordinal(50 * 100_000_000 * 2016 - 1).degree().to_string(),
      "0°2015′2015″4999999999‴"
    );
    assert_eq!(
      Ordinal(50 * 100_000_000 * 2016).degree().to_string(),
      "0°2016′0″0‴"
    );
    assert_eq!(
      Ordinal(50 * 100_000_000 * 2016 + 1).degree().to_string(),
      "0°2016′0″1‴"
    );
    assert_eq!(
      Ordinal(50 * 100_000_000 * 210000 - 1).degree().to_string(),
      "0°209999′335″4999999999‴"
    );
    assert_eq!(
      Ordinal(50 * 100_000_000 * 210000).degree().to_string(),
      "0°0′336″0‴"
    );
    assert_eq!(
      Ordinal(50 * 100_000_000 * 210000 + 1).degree().to_string(),
      "0°0′336″1‴"
    );
    assert_eq!(
      Ordinal(2067187500000000 - 1).degree().to_string(),
      "0°209999′2015″156249999‴"
    );
    assert_eq!(Ordinal(2067187500000000).degree().to_string(), "1°0′0″0‴");
    assert_eq!(
      Ordinal(2067187500000000 + 1).degree().to_string(),
      "1°0′0″1‴"
    );
  }

  #[test]
  fn period() {
    assert_eq!(Ordinal(0).period(), 0);
    assert_eq!(Ordinal(10080000000000).period(), 1);
    assert_eq!(Ordinal(2099999997689999).period(), 3437);
    assert_eq!(Ordinal(10075000000000).period(), 0);
    assert_eq!(Ordinal(10080000000000 - 1).period(), 0);
    assert_eq!(Ordinal(10080000000000).period(), 1);
    assert_eq!(Ordinal(10080000000000 + 1).period(), 1);
    assert_eq!(Ordinal(10085000000000).period(), 1);
    assert_eq!(Ordinal(2099999997689999).period(), 3437);
  }

  #[test]
  fn epoch() {
    assert_eq!(Ordinal(0).epoch(), 0);
    assert_eq!(Ordinal(1).epoch(), 0);
    assert_eq!(Ordinal(50 * 100_000_000 * 210000).epoch(), 1);
    assert_eq!(Ordinal(2099999997689999).epoch(), 32);
  }

  #[test]
  fn epoch_position() {
    assert_eq!(Epoch(0).starting_ordinal().epoch_position(), 0);
    assert_eq!((Epoch(0).starting_ordinal() + 100).epoch_position(), 100);
    assert_eq!(Epoch(1).starting_ordinal().epoch_position(), 0);
    assert_eq!(Epoch(2).starting_ordinal().epoch_position(), 0);
  }

  #[test]
  fn subsidy_position() {
    assert_eq!(Ordinal(0).third(), 0);
    assert_eq!(Ordinal(1).third(), 1);
    assert_eq!(
      Ordinal(Height(0).subsidy() - 1).third(),
      Height(0).subsidy() - 1
    );
    assert_eq!(Ordinal(Height(0).subsidy()).third(), 0);
    assert_eq!(Ordinal(Height(0).subsidy() + 1).third(), 1);
    assert_eq!(
      Ordinal(Epoch(1).starting_ordinal().n() + Epoch(1).subsidy()).third(),
      0
    );
    assert_eq!(Ordinal::LAST.third(), 0);
  }

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

    assert_eq!(Ordinal::SUPPLY, mined);
  }

  #[test]
  fn last() {
    assert_eq!(Ordinal::LAST, Ordinal::SUPPLY - 1);
  }

  #[test]
  fn eq() {
    assert_eq!(Ordinal(0), 0);
    assert_eq!(Ordinal(1), 1);
  }

  #[test]
  fn add() {
    assert_eq!(Ordinal(0) + 1, 1);
    assert_eq!(Ordinal(1) + 100, 101);
  }

  #[test]
  fn add_assign() {
    let mut ordinal = Ordinal(0);
    ordinal += 1;
    assert_eq!(ordinal, 1);
    ordinal += 100;
    assert_eq!(ordinal, 101);
  }

  fn parse(s: &str) -> Result<Ordinal, String> {
    s.parse::<Ordinal>().map_err(|e| e.to_string())
  }

  #[test]
  fn from_str_decimal() {
    assert_eq!(parse("0.0").unwrap(), 0);
    assert_eq!(parse("0.1").unwrap(), 1);
    assert_eq!(parse("1.0").unwrap(), 50 * 100_000_000);
    assert_eq!(parse("6929999.0").unwrap(), 2099999997689999);
    assert!(parse("0.5000000000").is_err());
    assert!(parse("6930000.0").is_err());
  }

  #[test]
  fn from_str_degree() {
    assert_eq!(parse("0°0′0″0‴").unwrap(), 0);
    assert_eq!(parse("0°0′0″").unwrap(), 0);
    assert_eq!(parse("0°0′0″1‴").unwrap(), 1);
    assert_eq!(parse("0°2015′2015″0‴").unwrap(), 10075000000000);
    assert_eq!(parse("0°2016′0″0‴").unwrap(), 10080000000000);
    assert_eq!(parse("0°2017′1″0‴").unwrap(), 10085000000000);
    assert_eq!(parse("0°2016′0″1‴").unwrap(), 10080000000001);
    assert_eq!(parse("0°2017′1″1‴").unwrap(), 10085000000001);
    assert_eq!(parse("0°209999′335″0‴").unwrap(), 1049995000000000);
    assert_eq!(parse("0°0′336″0‴").unwrap(), 1050000000000000);
    assert_eq!(parse("0°0′672″0‴").unwrap(), 1575000000000000);
    assert_eq!(parse("0°209999′1007″0‴").unwrap(), 1837498750000000);
    assert_eq!(parse("0°0′1008″0‴").unwrap(), 1837500000000000);
    assert_eq!(parse("1°0′0″0‴").unwrap(), 2067187500000000);
    assert_eq!(parse("2°0′0″0‴").unwrap(), 2099487304530000);
    assert_eq!(parse("3°0′0″0‴").unwrap(), 2099991988080000);
    assert_eq!(parse("4°0′0″0‴").unwrap(), 2099999873370000);
    assert_eq!(parse("5°0′0″0‴").unwrap(), 2099999996220000);
    assert_eq!(parse("5°0′336″0‴").unwrap(), 2099999997060000);
    assert_eq!(parse("5°0′672″0‴").unwrap(), 2099999997480000);
    assert_eq!(parse("5°1′673″0‴").unwrap(), 2099999997480001);
    assert_eq!(parse("5°209999′1007″0‴").unwrap(), 2099999997689999);
  }

  #[test]
  fn from_str_number() {
    assert_eq!(parse("0").unwrap(), 0);
    assert_eq!(parse("2099999997689999").unwrap(), 2099999997689999);
    assert!(parse("2099999997690000").is_err());
  }

  #[test]
  fn from_str_degree_invalid_cycle_number() {
    assert!(parse("5°0′0″0‴").is_ok());
    assert!(parse("6°0′0″0‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_epoch_offset() {
    assert!(parse("0°209999′335″0‴").is_ok());
    assert!(parse("0°210000′336″0‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_period_offset() {
    assert!(parse("0°2015′2015″0‴").is_ok());
    assert!(parse("0°2016′2016″0‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_block_offset() {
    assert!(parse("0°0′0″4999999999‴").is_ok());
    assert!(parse("0°0′0″5000000000‴").is_err());
    assert!(parse("0°209999′335″4999999999‴").is_ok());
    assert!(parse("0°0′336″4999999999‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_period_block_relationship() {
    assert!(parse("0°2015′2015″0‴").is_ok());
    assert!(parse("0°2016′0″0‴").is_ok());
    assert!(parse("0°2016′1″0‴").is_err());
  }

  #[test]
  fn from_str_degree_post_distribution() {
    assert!(parse("5°209999′1007″0‴").is_ok());
    assert!(parse("5°0′1008″0‴").is_err());
  }

  #[test]
  fn from_str_name() {
    assert_eq!(parse("nvtdijuwxlp").unwrap(), 0);
    assert_eq!(parse("a").unwrap(), 2099999997689999);
    assert!(parse("(").is_err());
    assert!(parse("").is_err());
    assert!(parse("nvtdijuwxlq").is_err());
  }

  #[test]
  fn cycle() {
    assert_eq!(Epoch::BLOCKS * CYCLE_EPOCHS % PERIOD_BLOCKS, 0);

    for i in 1..CYCLE_EPOCHS {
      assert_ne!(i * Epoch::BLOCKS % PERIOD_BLOCKS, 0);
    }

    assert_eq!(CYCLE_EPOCHS * Epoch::BLOCKS % PERIOD_BLOCKS, 0);

    assert_eq!(Ordinal(0).cycle(), 0);
    assert_eq!(Ordinal(2067187500000000 - 1).cycle(), 0);
    assert_eq!(Ordinal(2067187500000000).cycle(), 1);
    assert_eq!(Ordinal(2067187500000000 + 1).cycle(), 1);
  }

  #[test]
  fn rarity() {
    assert_eq!(Ordinal(0).rarity(), "mythic");
    assert_eq!(Ordinal(1).rarity(), "common");

    assert_eq!(Ordinal(50 * 100_000_000 - 1).rarity(), "common");
    assert_eq!(Ordinal(50 * 100_000_000).rarity(), "uncommon");
    assert_eq!(Ordinal(50 * 100_000_000 + 1).rarity(), "common");

    assert_eq!(Ordinal(50 * 100_000_000 * 2016 - 1).rarity(), "common");
    assert_eq!(Ordinal(50 * 100_000_000 * 2016).rarity(), "rare");
    assert_eq!(Ordinal(50 * 100_000_000 * 2016 + 1).rarity(), "common");

    assert_eq!(Ordinal(50 * 100_000_000 * 210000 - 1).rarity(), "common");
    assert_eq!(Ordinal(50 * 100_000_000 * 210000).rarity(), "epic");
    assert_eq!(Ordinal(50 * 100_000_000 * 210000 + 1).rarity(), "common");

    assert_eq!(Ordinal(2067187500000000 - 1).rarity(), "common");
    assert_eq!(Ordinal(2067187500000000).rarity(), "legendary");
    assert_eq!(Ordinal(2067187500000000 + 1).rarity(), "common");
  }

  #[test]
  fn third() {
    assert_eq!(Ordinal(0).third(), 0);
    assert_eq!(Ordinal(50 * 100_000_000 - 1).third(), 4999999999);
    assert_eq!(Ordinal(50 * 100_000_000).third(), 0);
    assert_eq!(Ordinal(50 * 100_000_000 + 1).third(), 1);
  }
}
