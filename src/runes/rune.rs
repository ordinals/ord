use super::*;

#[derive(Default, Debug, PartialEq, Copy, Clone, PartialOrd, Ord, Eq)]
pub struct Rune(pub u128);

impl Rune {
  pub(crate) fn minimum_at_height(height: Height) -> Self {
    let length = 13u64
      .saturating_sub(height.0 / (DIFFCHANGE_INTERVAL * 2))
      .max(1);

    let mut rune = 0u128;
    for i in 0..length {
      if i > 0 {
        rune += 1;
      }
      rune *= 26;
    }

    Rune(rune)
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
      x *= 26;
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
  fn minimum_at_height() {
    #[track_caller]
    fn case(height: u64, minimum: &str) {
      assert_eq!(Rune::minimum_at_height(Height(height)).to_string(), minimum);
    }

    case(2016 * 2 * 00 + 0, "AAAAAAAAAAAAA");
    case(2016 * 2 * 00 + 1, "AAAAAAAAAAAAA");
    case(2016 * 2 * 01 - 1, "AAAAAAAAAAAAA");
    case(2016 * 2 * 01 + 0, "AAAAAAAAAAAA");
    case(2016 * 2 * 01 + 1, "AAAAAAAAAAAA");
    case(2016 * 2 * 02 - 1, "AAAAAAAAAAAA");
    case(2016 * 2 * 02 + 0, "AAAAAAAAAAA");
    case(2016 * 2 * 02 + 1, "AAAAAAAAAAA");
    case(2016 * 2 * 03 - 1, "AAAAAAAAAAA");
    case(2016 * 2 * 03 + 0, "AAAAAAAAAA");
    case(2016 * 2 * 03 + 1, "AAAAAAAAAA");
    case(2016 * 2 * 04 - 1, "AAAAAAAAAA");
    case(2016 * 2 * 04 + 0, "AAAAAAAAA");
    case(2016 * 2 * 04 + 1, "AAAAAAAAA");
    case(2016 * 2 * 05 - 1, "AAAAAAAAA");
    case(2016 * 2 * 05 + 0, "AAAAAAAA");
    case(2016 * 2 * 05 + 1, "AAAAAAAA");
    case(2016 * 2 * 06 - 1, "AAAAAAAA");
    case(2016 * 2 * 06 + 0, "AAAAAAA");
    case(2016 * 2 * 06 + 1, "AAAAAAA");
    case(2016 * 2 * 07 - 1, "AAAAAAA");
    case(2016 * 2 * 07 + 0, "AAAAAA");
    case(2016 * 2 * 07 + 1, "AAAAAA");
    case(2016 * 2 * 08 - 1, "AAAAAA");
    case(2016 * 2 * 08 + 0, "AAAAA");
    case(2016 * 2 * 08 + 1, "AAAAA");
    case(2016 * 2 * 09 - 1, "AAAAA");
    case(2016 * 2 * 09 + 0, "AAAA");
    case(2016 * 2 * 09 + 1, "AAAA");
    case(2016 * 2 * 10 - 1, "AAAA");
    case(2016 * 2 * 10 + 0, "AAA");
    case(2016 * 2 * 10 + 1, "AAA");
    case(2016 * 2 * 11 - 1, "AAA");
    case(2016 * 2 * 11 + 0, "AA");
    case(2016 * 2 * 11 + 1, "AA");
    case(2016 * 2 * 12 - 1, "AA");
    case(2016 * 2 * 12 + 0, "A");
    case(2016 * 2 * 12 + 1, "A");
    case(2016 * 2 * 13 - 1, "A");
    case(2016 * 2 * 13 + 0, "A");
    case(2016 * 2 * 13 + 1, "A");
    case(u64::max_value(), "A");
  }

  #[test]
  fn serde() {
    let rune = Rune(0);
    let json = "\"A\"";
    assert_eq!(serde_json::to_string(&rune).unwrap(), json);
    assert_eq!(serde_json::from_str::<Rune>(json).unwrap(), rune);
  }
}
