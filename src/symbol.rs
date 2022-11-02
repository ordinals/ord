use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Symbol(u64);

impl FromStr for Symbol {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut n = 0u128;
    for c in s.chars() {
      let value = (c as u128)
        .checked_sub('A' as u128)
        .filter(|x| *x < 26)
        .with_context(|| format!("invalid symbol character '{c}'"))?
        + 1;

      n = n.checked_mul(26).context("overflow")? + value;
    }
    Ok(Symbol((n - 1) as u64))
  }
}

impl Display for Symbol {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut x = self.0 as u128 + 1;
    let mut name = String::new();
    while x > 0 {
      name.push(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
          .chars()
          .nth(((x - 1) % 26) as usize)
          .unwrap(),
      );
      x = (x - 1) / 26;
    }
    write!(f, "{}", name.chars().rev().collect::<String>())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str_low_character_error() {
    assert_eq!(
      Symbol::from_str("@").unwrap_err().to_string(),
      "invalid symbol character '@'"
    );
  }

  #[test]
  fn from_str_high_character_error() {
    assert_eq!(
      Symbol::from_str("]").unwrap_err().to_string(),
      "invalid symbol character ']'"
    );
  }

  #[test]
  #[ignore]
  fn from_str_overflow() {
    assert_eq!(
      Symbol::from_str("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        .unwrap_err()
        .to_string(),
      "overflow"
    );
  }

  #[test]
  fn from_str_ok() {
    assert_eq!(Symbol::from_str("A").unwrap(), Symbol(0));
    assert_eq!(Symbol::from_str("Z").unwrap(), Symbol(25));
    assert_eq!(Symbol::from_str("AA").unwrap(), Symbol(26));
    assert_eq!(Symbol::from_str("AB").unwrap(), Symbol(27));
    assert_eq!(Symbol::from_str("AZ").unwrap(), Symbol(51));
    assert_eq!(
      Symbol::from_str("GKGWBYLWRXTLPP").unwrap(),
      Symbol(u64::MAX)
    );
  }

  #[test]
  fn display() {
    assert_eq!(Symbol(0).to_string(), "A");
    assert_eq!(Symbol(25).to_string(), "Z");
    assert_eq!(Symbol(26).to_string(), "AA");
    assert_eq!(Symbol(27).to_string(), "AB");
    assert_eq!(Symbol(51).to_string(), "AZ");
    assert_eq!(Symbol(52).to_string(), "BA");
    assert_eq!(Symbol(u64::MAX).to_string(), "GKGWBYLWRXTLPP");
  }
}
