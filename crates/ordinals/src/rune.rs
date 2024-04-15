use super::*;

#[derive(
  Default, Debug, PartialEq, Copy, Clone, PartialOrd, Ord, Eq, DeserializeFromStr, SerializeDisplay,
)]
pub struct Rune(pub u128);

impl Rune {
  const RESERVED: u128 = 6402364363415443603228541259936211926;

  const STEPS: &'static [u128] = &[
    0,
    26,
    702,
    18278,
    475254,
    12356630,
    321272406,
    8353082582,
    217180147158,
    5646683826134,
    146813779479510,
    3817158266467286,
    99246114928149462,
    2580398988131886038,
    67090373691429037014,
    1744349715977154962390,
    45353092615406029022166,
    1179180408000556754576342,
    30658690608014475618984918,
    797125955808376366093607894,
    20725274851017785518433805270,
    538857146126462423479278937046,
    14010285799288023010461252363222,
    364267430781488598271992561443798,
    9470953200318703555071806597538774,
    246244783208286292431866971536008150,
    6402364363415443603228541259936211926,
    166461473448801533683942072758341510102,
  ];

  pub fn n(self) -> u128 {
    self.0
  }

  pub fn first_rune_height(network: Network) -> u32 {
    SUBSIDY_HALVING_INTERVAL
      * match network {
        Network::Bitcoin => 4,
        Network::Regtest => 0,
        Network::Signet => 0,
        Network::Testnet => 12,
        _ => 0,
      }
  }

  pub fn minimum_at_height(chain: Network, height: Height) -> Self {
    let offset = height.0.saturating_add(1);

    const INTERVAL: u32 = SUBSIDY_HALVING_INTERVAL / 12;

    let start = Self::first_rune_height(chain);

    let end = start + SUBSIDY_HALVING_INTERVAL;

    if offset < start {
      return Rune(Self::STEPS[12]);
    }

    if offset >= end {
      return Rune(0);
    }

    let progress = offset.saturating_sub(start);

    let length = 12u32.saturating_sub(progress / INTERVAL);

    let end = Self::STEPS[usize::try_from(length - 1).unwrap()];

    let start = Self::STEPS[usize::try_from(length).unwrap()];

    let remainder = u128::from(progress % INTERVAL);

    Rune(start - ((start - end) * remainder / u128::from(INTERVAL)))
  }

  pub fn is_reserved(self) -> bool {
    self.0 >= Self::RESERVED
  }

  pub fn reserved(block: u64, tx: u32) -> Self {
    Self(
      Self::RESERVED
        .checked_add(u128::from(block) << 32 | u128::from(tx))
        .unwrap(),
    )
  }

  pub fn commitment(self) -> Vec<u8> {
    let bytes = self.0.to_le_bytes();

    let mut end = bytes.len();

    while end > 0 && bytes[end - 1] == 0 {
      end -= 1;
    }

    bytes[..end].into()
  }
}

impl Display for Rune {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut n = self.0;
    if n == u128::MAX {
      return write!(f, "BCGDENLQRQWDSLRUGSNLBTMFIJAV");
    }

    n += 1;
    let mut symbol = String::new();
    while n > 0 {
      symbol.push(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
          .chars()
          .nth(((n - 1) % 26) as usize)
          .unwrap(),
      );
      n = (n - 1) / 26;
    }

    for c in symbol.chars().rev() {
      write!(f, "{c}")?;
    }

    Ok(())
  }
}

impl FromStr for Rune {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Error> {
    let mut x = 0u128;
    for (i, c) in s.chars().enumerate() {
      if i > 0 {
        x = x.checked_add(1).ok_or(Error::Range)?;
      }
      x = x.checked_mul(26).ok_or(Error::Range)?;
      match c {
        'A'..='Z' => {
          x = x.checked_add(c as u128 - 'A' as u128).ok_or(Error::Range)?;
        }
        _ => return Err(Error::Character(c)),
      }
    }
    Ok(Rune(x))
  }
}

#[derive(Debug, PartialEq)]
pub enum Error {
  Character(char),
  Range,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Character(c) => write!(f, "invalid character `{c}`"),
      Self::Range => write!(f, "name out of range"),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    fn case(n: u128, s: &str) {
      assert_eq!(Rune(n).to_string(), s);
      assert_eq!(s.parse::<Rune>().unwrap(), Rune(n));
    }

    case(0, "A");
    case(1, "B");
    case(2, "C");
    case(3, "D");
    case(4, "E");
    case(5, "F");
    case(6, "G");
    case(7, "H");
    case(8, "I");
    case(9, "J");
    case(10, "K");
    case(11, "L");
    case(12, "M");
    case(13, "N");
    case(14, "O");
    case(15, "P");
    case(16, "Q");
    case(17, "R");
    case(18, "S");
    case(19, "T");
    case(20, "U");
    case(21, "V");
    case(22, "W");
    case(23, "X");
    case(24, "Y");
    case(25, "Z");
    case(26, "AA");
    case(27, "AB");
    case(51, "AZ");
    case(52, "BA");
    case(u128::MAX - 2, "BCGDENLQRQWDSLRUGSNLBTMFIJAT");
    case(u128::MAX - 1, "BCGDENLQRQWDSLRUGSNLBTMFIJAU");
    case(u128::MAX, "BCGDENLQRQWDSLRUGSNLBTMFIJAV");
  }

  #[test]
  fn from_str_error() {
    assert_eq!(
      "BCGDENLQRQWDSLRUGSNLBTMFIJAW".parse::<Rune>().unwrap_err(),
      Error::Range,
    );
    assert_eq!(
      "BCGDENLQRQWDSLRUGSNLBTMFIJAVX".parse::<Rune>().unwrap_err(),
      Error::Range,
    );
    assert_eq!("x".parse::<Rune>().unwrap_err(), Error::Character('x'));
  }

  #[test]
  #[allow(clippy::identity_op)]
  #[allow(clippy::erasing_op)]
  #[allow(clippy::zero_prefixed_literal)]
  fn mainnet_minimum_at_height() {
    #[track_caller]
    fn case(height: u32, minimum: &str) {
      assert_eq!(
        Rune::minimum_at_height(Network::Bitcoin, Height(height)).to_string(),
        minimum,
      );
    }

    const START: u32 = SUBSIDY_HALVING_INTERVAL * 4;
    const END: u32 = START + SUBSIDY_HALVING_INTERVAL;
    const INTERVAL: u32 = SUBSIDY_HALVING_INTERVAL / 12;

    case(0, "AAAAAAAAAAAAA");
    case(START / 2, "AAAAAAAAAAAAA");
    case(START, "ZZYZXBRKWXVA");
    case(START + 1, "ZZXZUDIVTVQA");
    case(END - 1, "A");
    case(END, "A");
    case(END + 1, "A");
    case(u32::MAX, "A");

    case(START + INTERVAL * 00 - 1, "AAAAAAAAAAAAA");
    case(START + INTERVAL * 00 + 0, "ZZYZXBRKWXVA");
    case(START + INTERVAL * 00 + 1, "ZZXZUDIVTVQA");

    case(START + INTERVAL * 01 - 1, "AAAAAAAAAAAA");
    case(START + INTERVAL * 01 + 0, "ZZYZXBRKWXV");
    case(START + INTERVAL * 01 + 1, "ZZXZUDIVTVQ");

    case(START + INTERVAL * 02 - 1, "AAAAAAAAAAA");
    case(START + INTERVAL * 02 + 0, "ZZYZXBRKWY");
    case(START + INTERVAL * 02 + 1, "ZZXZUDIVTW");

    case(START + INTERVAL * 03 - 1, "AAAAAAAAAA");
    case(START + INTERVAL * 03 + 0, "ZZYZXBRKX");
    case(START + INTERVAL * 03 + 1, "ZZXZUDIVU");

    case(START + INTERVAL * 04 - 1, "AAAAAAAAA");
    case(START + INTERVAL * 04 + 0, "ZZYZXBRL");
    case(START + INTERVAL * 04 + 1, "ZZXZUDIW");

    case(START + INTERVAL * 05 - 1, "AAAAAAAA");
    case(START + INTERVAL * 05 + 0, "ZZYZXBS");
    case(START + INTERVAL * 05 + 1, "ZZXZUDJ");

    case(START + INTERVAL * 06 - 1, "AAAAAAA");
    case(START + INTERVAL * 06 + 0, "ZZYZXC");
    case(START + INTERVAL * 06 + 1, "ZZXZUE");

    case(START + INTERVAL * 07 - 1, "AAAAAA");
    case(START + INTERVAL * 07 + 0, "ZZYZY");
    case(START + INTERVAL * 07 + 1, "ZZXZV");

    case(START + INTERVAL * 08 - 1, "AAAAA");
    case(START + INTERVAL * 08 + 0, "ZZZA");
    case(START + INTERVAL * 08 + 1, "ZZYA");

    case(START + INTERVAL * 09 - 1, "AAAA");
    case(START + INTERVAL * 09 + 0, "ZZZ");
    case(START + INTERVAL * 09 + 1, "ZZY");

    case(START + INTERVAL * 10 - 2, "AAC");
    case(START + INTERVAL * 10 - 1, "AAA");
    case(START + INTERVAL * 10 + 0, "AAA");
    case(START + INTERVAL * 10 + 1, "AAA");

    case(START + INTERVAL * 10 + INTERVAL / 2, "NA");

    case(START + INTERVAL * 11 - 2, "AB");
    case(START + INTERVAL * 11 - 1, "AA");
    case(START + INTERVAL * 11 + 0, "AA");
    case(START + INTERVAL * 11 + 1, "AA");

    case(START + INTERVAL * 11 + INTERVAL / 2, "N");

    case(START + INTERVAL * 12 - 2, "B");
    case(START + INTERVAL * 12 - 1, "A");
    case(START + INTERVAL * 12 + 0, "A");
    case(START + INTERVAL * 12 + 1, "A");
  }

  #[test]
  fn minimum_at_height() {
    #[track_caller]
    fn case(network: Network, height: u32, minimum: &str) {
      assert_eq!(
        Rune::minimum_at_height(network, Height(height)).to_string(),
        minimum,
      );
    }

    case(Network::Testnet, 0, "AAAAAAAAAAAAA");
    case(
      Network::Testnet,
      SUBSIDY_HALVING_INTERVAL * 12 - 1,
      "AAAAAAAAAAAAA",
    );
    case(
      Network::Testnet,
      SUBSIDY_HALVING_INTERVAL * 12,
      "ZZYZXBRKWXVA",
    );
    case(
      Network::Testnet,
      SUBSIDY_HALVING_INTERVAL * 12 + 1,
      "ZZXZUDIVTVQA",
    );

    case(Network::Signet, 0, "ZZYZXBRKWXVA");
    case(Network::Signet, 1, "ZZXZUDIVTVQA");

    case(Network::Regtest, 0, "ZZYZXBRKWXVA");
    case(Network::Regtest, 1, "ZZXZUDIVTVQA");
  }

  #[test]
  fn serde() {
    let rune = Rune(0);
    let json = "\"A\"";
    assert_eq!(serde_json::to_string(&rune).unwrap(), json);
    assert_eq!(serde_json::from_str::<Rune>(json).unwrap(), rune);
  }

  #[test]
  fn reserved() {
    assert_eq!(
      Rune::RESERVED,
      "AAAAAAAAAAAAAAAAAAAAAAAAAAA".parse::<Rune>().unwrap().0,
    );

    assert_eq!(Rune::reserved(0, 0), Rune(Rune::RESERVED));
    assert_eq!(Rune::reserved(0, 1), Rune(Rune::RESERVED + 1));
    assert_eq!(Rune::reserved(1, 0), Rune(Rune::RESERVED + (1 << 32)));
    assert_eq!(Rune::reserved(1, 1), Rune(Rune::RESERVED + (1 << 32) + 1));
    assert_eq!(
      Rune::reserved(u64::MAX, u32::MAX),
      Rune(Rune::RESERVED + (u128::from(u64::MAX) << 32 | u128::from(u32::MAX))),
    );
  }

  #[test]
  fn is_reserved() {
    #[track_caller]
    fn case(rune: &str, reserved: bool) {
      assert_eq!(rune.parse::<Rune>().unwrap().is_reserved(), reserved);
    }

    case("A", false);
    case("ZZZZZZZZZZZZZZZZZZZZZZZZZZ", false);
    case("AAAAAAAAAAAAAAAAAAAAAAAAAAA", true);
    case("AAAAAAAAAAAAAAAAAAAAAAAAAAB", true);
    case("BCGDENLQRQWDSLRUGSNLBTMFIJAV", true);
  }

  #[test]
  fn steps() {
    for i in 0.. {
      match "A".repeat(i + 1).parse::<Rune>() {
        Ok(rune) => assert_eq!(Rune(Rune::STEPS[i]), rune),
        Err(_) => {
          assert_eq!(Rune::STEPS.len(), i);
          break;
        }
      }
    }
  }

  #[test]
  fn commitment() {
    #[track_caller]
    fn case(rune: u128, bytes: &[u8]) {
      assert_eq!(Rune(rune).commitment(), bytes);
    }

    case(0, &[]);
    case(1, &[1]);
    case(255, &[255]);
    case(256, &[0, 1]);
    case(65535, &[255, 255]);
    case(65536, &[0, 0, 1]);
    case(u128::MAX, &[255; 16]);
  }
}
