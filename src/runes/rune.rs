use super::*;

#[derive(Default, Debug, PartialEq, Copy, Clone, PartialOrd, Ord, Eq)]
pub struct Rune(pub u128);

impl Rune {
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

  pub(crate) fn minimum_at_height(chain: Chain, height: Height) -> Self {
    let start = chain.first_rune_height();

    let end = start + SUBSIDY_HALVING_INTERVAL;

    const INTERVAL: u32 = SUBSIDY_HALVING_INTERVAL / 12;

    if height.0 < start {
      return Rune(Self::STEPS[12]);
    }

    if height.0 >= end {
      return Rune(0);
    }

    let progress = height.0.saturating_sub(start);

    let length = 12u32.saturating_sub(progress / INTERVAL);

    let end = Self::STEPS[usize::try_from(length - 1).unwrap()];

    let start = Self::STEPS[usize::try_from(length).unwrap()];

    let remainder = u128::from(progress % INTERVAL);

    Rune(start - ((start - end) * remainder / u128::from(INTERVAL)))
  }
}

impl Serialize for Rune {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for Rune {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}

impl Display for Rune {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut n = self.0;
    if n == u128::max_value() {
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
  type Err = crate::Error;

  fn from_str(s: &str) -> crate::Result<Self> {
    let mut x = 0u128;
    for (i, c) in s.chars().enumerate() {
      if i > 0 {
        x += 1;
      }
      x = x.checked_mul(26).ok_or_else(|| anyhow!("out of range"))?;
      match c {
        'A'..='Z' => {
          x = x
            .checked_add(c as u128 - 'A' as u128)
            .ok_or_else(|| anyhow!("out of range"))?;
        }
        _ => bail!("invalid character in rune name: {c}"),
      }
    }
    Ok(Rune(x))
  }
}

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
    case(u128::max_value() - 2, "BCGDENLQRQWDSLRUGSNLBTMFIJAT");
    case(u128::max_value() - 1, "BCGDENLQRQWDSLRUGSNLBTMFIJAU");
    case(u128::max_value(), "BCGDENLQRQWDSLRUGSNLBTMFIJAV");
  }

  #[test]
  fn from_str_out_of_range() {
    "BCGDENLQRQWDSLRUGSNLBTMFIJAW".parse::<Rune>().unwrap_err();
  }

  #[test]
  #[allow(clippy::identity_op)]
  #[allow(clippy::erasing_op)]
  #[allow(clippy::zero_prefixed_literal)]
  fn mainnet_minimum_at_height() {
    #[track_caller]
    fn case(height: u32, minimum: &str) {
      assert_eq!(
        Rune::minimum_at_height(Chain::Mainnet, Height(height)).to_string(),
        minimum,
      );
    }

    const START: u32 = SUBSIDY_HALVING_INTERVAL * 4;
    const END: u32 = START + SUBSIDY_HALVING_INTERVAL;
    const STEP: u32 = (END - START) / 12;

    case(0, "AAAAAAAAAAAAA");
    case(START / 2, "AAAAAAAAAAAAA");
    case(START, "AAAAAAAAAAAAA");
    case(START + 1, "ZZYZXBRKWXVA");
    case(END - 1, "B");
    case(END, "A");
    case(END + 1, "A");
    case(u32::max_value(), "A");

    case(START + STEP * 00 - 1, "AAAAAAAAAAAAA");
    case(START + STEP * 00 + 0, "AAAAAAAAAAAAA");
    case(START + STEP * 00 + 1, "ZZYZXBRKWXVA");

    case(START + STEP * 01 - 1, "AABACYIPDCFB");
    case(START + STEP * 01 + 0, "AAAAAAAAAAAA");
    case(START + STEP * 01 + 1, "ZZYZXBRKWXV");

    case(START + STEP * 02 - 1, "AABACYIPDCG");
    case(START + STEP * 02 + 0, "AAAAAAAAAAA");
    case(START + STEP * 02 + 1, "ZZYZXBRKWY");

    case(START + STEP * 03 - 1, "AABACYIPDD");
    case(START + STEP * 03 + 0, "AAAAAAAAAA");
    case(START + STEP * 03 + 1, "ZZYZXBRKX");

    case(START + STEP * 04 - 1, "AABACYIPE");
    case(START + STEP * 04 + 0, "AAAAAAAAA");
    case(START + STEP * 04 + 1, "ZZYZXBRL");

    case(START + STEP * 05 - 1, "AABACYIQ");
    case(START + STEP * 05 + 0, "AAAAAAAA");
    case(START + STEP * 05 + 1, "ZZYZXBS");

    case(START + STEP * 06 - 1, "AABACYJ");
    case(START + STEP * 06 + 0, "AAAAAAA");
    case(START + STEP * 06 + 1, "ZZYZXC");

    case(START + STEP * 07 - 1, "AABACZ");
    case(START + STEP * 07 + 0, "AAAAAA");
    case(START + STEP * 07 + 1, "ZZYZY");

    case(START + STEP * 08 - 1, "AABAD");
    case(START + STEP * 08 + 0, "AAAAA");
    case(START + STEP * 08 + 1, "ZZZA");

    case(START + STEP * 09 - 1, "AABB");
    case(START + STEP * 09 + 0, "AAAA");
    case(START + STEP * 09 + 1, "ZZZ");

    case(START + STEP * 10 - 1, "AAC");
    case(START + STEP * 10 + 0, "AAA");
    case(START + STEP * 10 + 1, "AAA");

    case(START + STEP * 10 + STEP / 2, "NA");

    case(START + STEP * 11 - 1, "AB");
    case(START + STEP * 11 + 0, "AA");
    case(START + STEP * 11 + 1, "AA");

    case(START + STEP * 11 + STEP / 2, "N");

    case(START + STEP * 12 - 1, "B");
    case(START + STEP * 12 + 0, "A");
    case(START + STEP * 12 + 1, "A");
  }

  #[test]
  fn minimum_at_height() {
    #[track_caller]
    fn case(chain: Chain, height: u32, minimum: &str) {
      assert_eq!(
        Rune::minimum_at_height(chain, Height(height)).to_string(),
        minimum,
      );
    }

    case(Chain::Testnet, 0, "AAAAAAAAAAAAA");
    case(
      Chain::Testnet,
      SUBSIDY_HALVING_INTERVAL * 12,
      "AAAAAAAAAAAAA",
    );
    case(
      Chain::Testnet,
      SUBSIDY_HALVING_INTERVAL * 12 + 1,
      "ZZYZXBRKWXVA",
    );

    case(Chain::Signet, 0, "AAAAAAAAAAAAA");
    case(Chain::Signet, 1, "ZZYZXBRKWXVA");

    case(Chain::Regtest, 0, "AAAAAAAAAAAAA");
    case(Chain::Regtest, 1, "ZZYZXBRKWXVA");
  }

  #[test]
  fn serde() {
    let rune = Rune(0);
    let json = "\"A\"";
    assert_eq!(serde_json::to_string(&rune).unwrap(), json);
    assert_eq!(serde_json::from_str::<Rune>(json).unwrap(), rune);
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
}
