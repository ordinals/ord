use {super::*, std::num::ParseFloatError};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Sat(pub u64);

impl Sat {
  pub const LAST: Self = Self(Self::SUPPLY - 1);
  pub const SUPPLY: u64 = 2099999997690000;

  pub fn n(self) -> u64 {
    self.0
  }

  pub fn degree(self) -> Degree {
    self.into()
  }

  pub fn height(self) -> Height {
    self.epoch().starting_height()
      + u32::try_from(self.epoch_position() / self.epoch().subsidy()).unwrap()
  }

  pub fn cycle(self) -> u32 {
    Epoch::from(self).0 / CYCLE_EPOCHS
  }

  pub fn nineball(self) -> bool {
    self.n() >= 50 * COIN_VALUE * 9 && self.n() < 50 * COIN_VALUE * 10
  }

  pub fn palindrome(self) -> bool {
    let mut n = self.0;
    let mut reversed = 0;

    while n > 0 {
      reversed = reversed * 10 + n % 10;
      n /= 10;
    }

    self.0 == reversed
  }

  pub fn percentile(self) -> String {
    format!("{}%", (self.0 as f64 / Self::LAST.0 as f64) * 100.0)
  }

  pub fn epoch(self) -> Epoch {
    self.into()
  }

  pub fn period(self) -> u32 {
    self.height().n() / DIFFCHANGE_INTERVAL
  }

  pub fn third(self) -> u64 {
    self.epoch_position() % self.epoch().subsidy()
  }

  pub fn epoch_position(self) -> u64 {
    self.0 - self.epoch().starting_sat().0
  }

  pub fn decimal(self) -> DecimalSat {
    self.into()
  }

  pub fn rarity(self) -> Rarity {
    self.into()
  }

  /// Is this sat common or not?  Much faster than `Sat::rarity()`.
  pub fn common(self) -> bool {
    // The block rewards for epochs 0 through 9 are all multiples
    // of 9765625 (the epoch 9 reward), so any sat from epoch 9 or
    // earlier that isn't divisible by 9765625 is definitely common.
    if self < Epoch(10).starting_sat() && self.0 % Epoch(9).subsidy() != 0 {
      return true;
    }

    // Fall back to the full calculation.
    let epoch = self.epoch();
    (self.0 - epoch.starting_sat().0) % epoch.subsidy() != 0
  }

  pub fn coin(self) -> bool {
    self.n() % COIN_VALUE == 0
  }

  pub fn name(self) -> String {
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

  pub fn charms(self) -> u16 {
    let mut charms = 0;

    if self.nineball() {
      Charm::Nineball.set(&mut charms);
    }

    if self.palindrome() {
      Charm::Palindrome.set(&mut charms);
    }

    if self.coin() {
      Charm::Coin.set(&mut charms);
    }

    match self.rarity() {
      Rarity::Common => {}
      Rarity::Epic => Charm::Epic.set(&mut charms),
      Rarity::Legendary => Charm::Legendary.set(&mut charms),
      Rarity::Mythic => Charm::Mythic.set(&mut charms),
      Rarity::Rare => Charm::Rare.set(&mut charms),
      Rarity::Uncommon => Charm::Uncommon.set(&mut charms),
    }

    charms
  }

  fn from_name(s: &str) -> Result<Self, Error> {
    let mut x = 0;
    for c in s.chars() {
      match c {
        'a'..='z' => {
          x = x * 26 + c as u64 - 'a' as u64 + 1;
          if x > Self::SUPPLY {
            return Err(ErrorKind::NameRange.error(s));
          }
        }
        _ => return Err(ErrorKind::NameCharacter.error(s)),
      }
    }
    Ok(Sat(Self::SUPPLY - x))
  }

  fn from_degree(degree: &str) -> Result<Self, Error> {
    let (cycle_number, rest) = degree
      .split_once('°')
      .ok_or_else(|| ErrorKind::MissingDegree.error(degree))?;

    let cycle_number = cycle_number
      .parse::<u32>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?;

    let (epoch_offset, rest) = rest
      .split_once('′')
      .ok_or_else(|| ErrorKind::MissingMinute.error(degree))?;

    let epoch_offset = epoch_offset
      .parse::<u32>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?;

    if epoch_offset >= SUBSIDY_HALVING_INTERVAL {
      return Err(ErrorKind::EpochOffset.error(degree));
    }

    let (period_offset, rest) = rest
      .split_once('″')
      .ok_or_else(|| ErrorKind::MissingSecond.error(degree))?;

    let period_offset = period_offset
      .parse::<u32>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?;

    if period_offset >= DIFFCHANGE_INTERVAL {
      return Err(ErrorKind::PeriodOffset.error(degree));
    }

    let cycle_start_epoch = cycle_number * CYCLE_EPOCHS;

    const HALVING_INCREMENT: u32 = SUBSIDY_HALVING_INTERVAL % DIFFCHANGE_INTERVAL;

    // For valid degrees the relationship between epoch_offset and period_offset
    // will increment by 336 every halving.
    let relationship = period_offset + SUBSIDY_HALVING_INTERVAL * CYCLE_EPOCHS - epoch_offset;

    if relationship % HALVING_INCREMENT != 0 {
      return Err(ErrorKind::EpochPeriodMismatch.error(degree));
    }

    let epochs_since_cycle_start = relationship % DIFFCHANGE_INTERVAL / HALVING_INCREMENT;

    let epoch = cycle_start_epoch + epochs_since_cycle_start;

    let height = Height(epoch * SUBSIDY_HALVING_INTERVAL + epoch_offset);

    let (block_offset, rest) = match rest.split_once('‴') {
      Some((block_offset, rest)) => (
        block_offset
          .parse::<u64>()
          .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?,
        rest,
      ),
      None => (0, rest),
    };

    if !rest.is_empty() {
      return Err(ErrorKind::TrailingCharacters.error(degree));
    }

    if block_offset >= height.subsidy() {
      return Err(ErrorKind::BlockOffset.error(degree));
    }

    Ok(height.starting_sat() + block_offset)
  }

  fn from_decimal(decimal: &str) -> Result<Self, Error> {
    let (height, offset) = decimal
      .split_once('.')
      .ok_or_else(|| ErrorKind::MissingPeriod.error(decimal))?;

    let height = Height(
      height
        .parse()
        .map_err(|source| ErrorKind::ParseInt { source }.error(decimal))?,
    );

    let offset = offset
      .parse::<u64>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(decimal))?;

    if offset >= height.subsidy() {
      return Err(ErrorKind::BlockOffset.error(decimal));
    }

    Ok(height.starting_sat() + offset)
  }

  fn from_percentile(percentile: &str) -> Result<Self, Error> {
    if !percentile.ends_with('%') {
      return Err(ErrorKind::Percentile.error(percentile));
    }

    let percentile_string = percentile;

    let percentile = percentile[..percentile.len() - 1]
      .parse::<f64>()
      .map_err(|source| ErrorKind::ParseFloat { source }.error(percentile))?;

    if percentile < 0.0 {
      return Err(ErrorKind::Percentile.error(percentile_string));
    }

    let last = Sat::LAST.n() as f64;

    let n = (percentile / 100.0 * last).round();

    if n > last {
      return Err(ErrorKind::Percentile.error(percentile_string));
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Ok(Sat(n as u64))
  }
}

#[derive(Debug, Error)]
pub struct Error {
  input: String,
  kind: ErrorKind,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "failed to parse sat `{}`: {}", self.input, self.kind)
  }
}

#[derive(Debug, Error)]
pub enum ErrorKind {
  IntegerRange,
  NameRange,
  NameCharacter,
  Percentile,
  BlockOffset,
  MissingPeriod,
  TrailingCharacters,
  MissingDegree,
  MissingMinute,
  MissingSecond,
  PeriodOffset,
  EpochOffset,
  EpochPeriodMismatch,
  ParseInt { source: ParseIntError },
  ParseFloat { source: ParseFloatError },
}

impl ErrorKind {
  fn error(self, input: &str) -> Error {
    Error {
      input: input.to_string(),
      kind: self,
    }
  }
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::IntegerRange => write!(f, "invalid integer range"),
      Self::NameRange => write!(f, "invalid name range"),
      Self::NameCharacter => write!(f, "invalid character in name"),
      Self::Percentile => write!(f, "invalid percentile"),
      Self::BlockOffset => write!(f, "invalid block offset"),
      Self::MissingPeriod => write!(f, "missing period"),
      Self::TrailingCharacters => write!(f, "trailing character"),
      Self::MissingDegree => write!(f, "missing degree symbol"),
      Self::MissingMinute => write!(f, "missing minute symbol"),
      Self::MissingSecond => write!(f, "missing second symbol"),
      Self::PeriodOffset => write!(f, "invalid period offset"),
      Self::EpochOffset => write!(f, "invalid epoch offset"),
      Self::EpochPeriodMismatch => write!(
        f,
        "relationship between epoch offset and period offset must be multiple of 336"
      ),
      Self::ParseInt { source } => write!(f, "invalid integer: {source}"),
      Self::ParseFloat { source } => write!(f, "invalid float: {source}"),
    }
  }
}

impl PartialEq<u64> for Sat {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl PartialOrd<u64> for Sat {
  fn partial_cmp(&self, other: &u64) -> Option<cmp::Ordering> {
    self.0.partial_cmp(other)
  }
}

impl Add<u64> for Sat {
  type Output = Self;

  fn add(self, other: u64) -> Sat {
    Sat(self.0 + other)
  }
}

impl AddAssign<u64> for Sat {
  fn add_assign(&mut self, other: u64) {
    *self = Sat(self.0 + other);
  }
}

impl FromStr for Sat {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.chars().any(|c| c.is_ascii_lowercase()) {
      Self::from_name(s)
    } else if s.contains('°') {
      Self::from_degree(s)
    } else if s.contains('%') {
      Self::from_percentile(s)
    } else if s.contains('.') {
      Self::from_decimal(s)
    } else {
      let sat = Self(
        s.parse()
          .map_err(|source| ErrorKind::ParseInt { source }.error(s))?,
      );
      if sat > Self::LAST {
        Err(ErrorKind::IntegerRange.error(s))
      } else {
        Ok(sat)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn n() {
    assert_eq!(Sat(1).n(), 1);
    assert_eq!(Sat(100).n(), 100);
  }

  #[test]
  fn height() {
    assert_eq!(Sat(0).height(), 0);
    assert_eq!(Sat(1).height(), 0);
    assert_eq!(Sat(Epoch(0).subsidy()).height(), 1);
    assert_eq!(Sat(Epoch(0).subsidy() * 2).height(), 2);
    assert_eq!(
      Epoch(2).starting_sat().height(),
      SUBSIDY_HALVING_INTERVAL * 2
    );
    assert_eq!(Sat(50 * COIN_VALUE).height(), 1);
    assert_eq!(Sat(2099999997689999).height(), 6929999);
    assert_eq!(Sat(2099999997689998).height(), 6929998);
  }

  #[test]
  fn name() {
    assert_eq!(Sat(0).name(), "nvtdijuwxlp");
    assert_eq!(Sat(1).name(), "nvtdijuwxlo");
    assert_eq!(Sat(26).name(), "nvtdijuwxkp");
    assert_eq!(Sat(27).name(), "nvtdijuwxko");
    assert_eq!(Sat(2099999997689999).name(), "a");
    assert_eq!(Sat(2099999997689999 - 1).name(), "b");
    assert_eq!(Sat(2099999997689999 - 25).name(), "z");
    assert_eq!(Sat(2099999997689999 - 26).name(), "aa");
  }

  #[test]
  fn number() {
    assert_eq!(Sat(2099999997689999).n(), 2099999997689999);
  }

  #[test]
  fn degree() {
    assert_eq!(Sat(0).degree().to_string(), "0°0′0″0‴");
    assert_eq!(Sat(1).degree().to_string(), "0°0′0″1‴");
    assert_eq!(
      Sat(50 * COIN_VALUE - 1).degree().to_string(),
      "0°0′0″4999999999‴"
    );
    assert_eq!(Sat(50 * COIN_VALUE).degree().to_string(), "0°1′1″0‴");
    assert_eq!(Sat(50 * COIN_VALUE + 1).degree().to_string(), "0°1′1″1‴");
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(DIFFCHANGE_INTERVAL) - 1)
        .degree()
        .to_string(),
      "0°2015′2015″4999999999‴"
    );
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(DIFFCHANGE_INTERVAL))
        .degree()
        .to_string(),
      "0°2016′0″0‴"
    );
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(DIFFCHANGE_INTERVAL) + 1)
        .degree()
        .to_string(),
      "0°2016′0″1‴"
    );
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(SUBSIDY_HALVING_INTERVAL) - 1)
        .degree()
        .to_string(),
      "0°209999′335″4999999999‴"
    );
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(SUBSIDY_HALVING_INTERVAL))
        .degree()
        .to_string(),
      "0°0′336″0‴"
    );
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(SUBSIDY_HALVING_INTERVAL) + 1)
        .degree()
        .to_string(),
      "0°0′336″1‴"
    );
    assert_eq!(
      Sat(2067187500000000 - 1).degree().to_string(),
      "0°209999′2015″156249999‴"
    );
    assert_eq!(Sat(2067187500000000).degree().to_string(), "1°0′0″0‴");
    assert_eq!(Sat(2067187500000000 + 1).degree().to_string(), "1°0′0″1‴");
  }

  #[test]
  fn invalid_degree_bugfix() {
    // Break glass in case of emergency:
    // for height in 0..(2 * CYCLE_EPOCHS * Epoch::BLOCKS) {
    //   // 1054200000000000
    //   let expected = Height(height).starting_sat();
    //   // 0°1680′0″0‴
    //   let degree = expected.degree();
    //   // 2034637500000000
    //   let actual = degree.to_string().parse::<Sat>().unwrap();
    //   assert_eq!(
    //     actual, expected,
    //     "Sat at height {height} did not round-trip from degree {degree} successfully"
    //   );
    // }
    assert_eq!(Sat(1054200000000000).degree().to_string(), "0°1680′0″0‴");
    assert_eq!(parse("0°1680′0″0‴").unwrap(), 1054200000000000);
    assert_eq!(
      Sat(1914226250000000).degree().to_string(),
      "0°122762′794″0‴"
    );
    assert_eq!(parse("0°122762′794″0‴").unwrap(), 1914226250000000);
  }

  #[test]
  fn period() {
    assert_eq!(Sat(0).period(), 0);
    assert_eq!(Sat(10080000000000).period(), 1);
    assert_eq!(Sat(2099999997689999).period(), 3437);
    assert_eq!(Sat(10075000000000).period(), 0);
    assert_eq!(Sat(10080000000000 - 1).period(), 0);
    assert_eq!(Sat(10080000000000).period(), 1);
    assert_eq!(Sat(10080000000000 + 1).period(), 1);
    assert_eq!(Sat(10085000000000).period(), 1);
    assert_eq!(Sat(2099999997689999).period(), 3437);
  }

  #[test]
  fn epoch() {
    assert_eq!(Sat(0).epoch(), 0);
    assert_eq!(Sat(1).epoch(), 0);
    assert_eq!(
      Sat(50 * COIN_VALUE * u64::from(SUBSIDY_HALVING_INTERVAL)).epoch(),
      1
    );
    assert_eq!(Sat(2099999997689999).epoch(), 32);
  }

  #[test]
  fn epoch_position() {
    assert_eq!(Epoch(0).starting_sat().epoch_position(), 0);
    assert_eq!((Epoch(0).starting_sat() + 100).epoch_position(), 100);
    assert_eq!(Epoch(1).starting_sat().epoch_position(), 0);
    assert_eq!(Epoch(2).starting_sat().epoch_position(), 0);
  }

  #[test]
  fn subsidy_position() {
    assert_eq!(Sat(0).third(), 0);
    assert_eq!(Sat(1).third(), 1);
    assert_eq!(
      Sat(Height(0).subsidy() - 1).third(),
      Height(0).subsidy() - 1
    );
    assert_eq!(Sat(Height(0).subsidy()).third(), 0);
    assert_eq!(Sat(Height(0).subsidy() + 1).third(), 1);
    assert_eq!(
      Sat(Epoch(1).starting_sat().n() + Epoch(1).subsidy()).third(),
      0
    );
    assert_eq!(Sat::LAST.third(), 0);
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

    assert_eq!(Sat::SUPPLY, mined);
  }

  #[test]
  fn last() {
    assert_eq!(Sat::LAST, Sat::SUPPLY - 1);
  }

  #[test]
  fn eq() {
    assert_eq!(Sat(0), 0);
    assert_eq!(Sat(1), 1);
  }

  #[test]
  fn partial_ord() {
    assert!(Sat(1) > 0);
    assert!(Sat(0) < 1);
  }

  #[test]
  fn add() {
    assert_eq!(Sat(0) + 1, 1);
    assert_eq!(Sat(1) + 100, 101);
  }

  #[test]
  fn add_assign() {
    let mut sat = Sat(0);
    sat += 1;
    assert_eq!(sat, 1);
    sat += 100;
    assert_eq!(sat, 101);
  }

  fn parse(s: &str) -> Result<Sat, String> {
    s.parse::<Sat>().map_err(|e| e.to_string())
  }

  #[test]
  fn from_str_decimal() {
    assert_eq!(parse("0.0").unwrap(), 0);
    assert_eq!(parse("0.1").unwrap(), 1);
    assert_eq!(parse("1.0").unwrap(), 50 * COIN_VALUE);
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
    assert!(parse("0°0′336″0‴").is_ok());
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
    assert!(parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err());
  }

  #[test]
  fn cycle() {
    assert_eq!(
      SUBSIDY_HALVING_INTERVAL * CYCLE_EPOCHS % DIFFCHANGE_INTERVAL,
      0
    );

    for i in 1..CYCLE_EPOCHS {
      assert_ne!(i * SUBSIDY_HALVING_INTERVAL % DIFFCHANGE_INTERVAL, 0);
    }

    assert_eq!(
      CYCLE_EPOCHS * SUBSIDY_HALVING_INTERVAL % DIFFCHANGE_INTERVAL,
      0
    );

    assert_eq!(Sat(0).cycle(), 0);
    assert_eq!(Sat(2067187500000000 - 1).cycle(), 0);
    assert_eq!(Sat(2067187500000000).cycle(), 1);
    assert_eq!(Sat(2067187500000000 + 1).cycle(), 1);
  }

  #[test]
  fn third() {
    assert_eq!(Sat(0).third(), 0);
    assert_eq!(Sat(50 * COIN_VALUE - 1).third(), 4999999999);
    assert_eq!(Sat(50 * COIN_VALUE).third(), 0);
    assert_eq!(Sat(50 * COIN_VALUE + 1).third(), 1);
  }

  #[test]
  fn percentile() {
    assert_eq!(Sat(0).percentile(), "0%");
    assert_eq!(Sat(Sat::LAST.n() / 2).percentile(), "49.99999999999998%");
    assert_eq!(Sat::LAST.percentile(), "100%");
  }

  #[test]
  fn from_percentile() {
    "-1%".parse::<Sat>().unwrap_err();
    "101%".parse::<Sat>().unwrap_err();
  }

  #[test]
  fn percentile_round_trip() {
    #[track_caller]
    fn case(n: u64) {
      let expected = Sat(n);
      let actual = expected.percentile().parse::<Sat>().unwrap();
      assert_eq!(expected, actual);
    }

    for n in 0..1024 {
      case(n);
      case(Sat::LAST.n() / 2 + n);
      case(Sat::LAST.n() - n);
      case(Sat::LAST.n() / (n + 1));
    }
  }

  #[test]
  fn common() {
    #[track_caller]
    fn case(n: u64) {
      assert_eq!(Sat(n).common(), Sat(n).rarity() == Rarity::Common);
    }

    case(0);
    case(1);
    case(50 * COIN_VALUE - 1);
    case(50 * COIN_VALUE);
    case(50 * COIN_VALUE + 1);
    case(2067187500000000 - 1);
    case(2067187500000000);
    case(2067187500000000 + 1);
  }

  #[test]
  fn common_fast_path() {
    // Exhaustively test the Sat::common() fast path on every
    // uncommon sat.
    for height in 0..Epoch::FIRST_POST_SUBSIDY.starting_height().0 {
      let height = Height(height);
      assert!(!Sat::common(height.starting_sat()));
    }
  }

  #[test]
  fn coin() {
    assert!(Sat(0).coin());
    assert!(!Sat(COIN_VALUE - 1).coin());
    assert!(Sat(COIN_VALUE).coin());
    assert!(!Sat(COIN_VALUE + 1).coin());
  }

  #[test]
  fn nineball() {
    for height in 0..10 {
      let sat = Sat(height * 50 * COIN_VALUE);
      assert_eq!(
        sat.nineball(),
        sat.height() == 9,
        "nineball: {} height: {}",
        sat.nineball(),
        sat.height()
      );
    }
  }

  #[test]
  fn error_display() {
    assert_eq!(
      Error {
        input: "foo".into(),
        kind: ErrorKind::Percentile
      }
      .to_string(),
      "failed to parse sat `foo`: invalid percentile",
    );
  }

  #[test]
  fn palindrome() {
    assert!(Sat(0).palindrome());
    assert!(!Sat(10).palindrome());
    assert!(Sat(11).palindrome());
  }

  #[test]
  fn palindrome_charm() {
    assert!(Charm::Palindrome.is_set(Sat(0).charms()));
    assert!(!Charm::Palindrome.is_set(Sat(10).charms()));
    assert!(Charm::Palindrome.is_set(Sat(11).charms()));
  }
}
