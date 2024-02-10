use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, PartialOrd, Eq)]
pub struct SpacedRune {
  pub(crate) rune: Rune,
  pub(crate) spacers: u32,
}

impl FromStr for SpacedRune {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut rune = String::new();
    let mut spacers = 0u32;

    for c in s.chars() {
      match c {
        'A'..='Z' => rune.push(c),
        '.' | '•' => {
          let flag = 1 << rune.len().checked_sub(1).context("leading spacer")?;
          if spacers & flag != 0 {
            bail!("double spacer");
          }
          spacers |= flag;
        }
        _ => bail!("invalid character"),
      }
    }

    if 32 - spacers.leading_zeros() >= rune.len().try_into().unwrap() {
      bail!("trailing spacer")
    }

    Ok(SpacedRune {
      rune: rune.parse()?,
      spacers,
    })
  }
}

impl Display for SpacedRune {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let rune = self.rune.to_string();

    for (i, c) in rune.chars().enumerate() {
      write!(f, "{c}")?;

      if i < rune.len() - 1 && self.spacers & 1 << i != 0 {
        write!(f, "•")?;
      }
    }

    Ok(())
  }
}

impl Serialize for SpacedRune {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for SpacedRune {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    DeserializeFromStr::with(deserializer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!("A.B".parse::<SpacedRune>().unwrap().to_string(), "A•B");
    assert_eq!("A.B.C".parse::<SpacedRune>().unwrap().to_string(), "A•B•C");
    assert_eq!(
      SpacedRune {
        rune: Rune(0),
        spacers: 1
      }
      .to_string(),
      "A"
    );
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, rune: &str, spacers: u32) {
      assert_eq!(
        s.parse::<SpacedRune>().unwrap(),
        SpacedRune {
          rune: rune.parse().unwrap(),
          spacers
        },
      );
    }

    assert_eq!(
      ".A".parse::<SpacedRune>().unwrap_err().to_string(),
      "leading spacer",
    );

    assert_eq!(
      "A..B".parse::<SpacedRune>().unwrap_err().to_string(),
      "double spacer",
    );

    assert_eq!(
      "A.".parse::<SpacedRune>().unwrap_err().to_string(),
      "trailing spacer",
    );

    assert_eq!(
      "Ax".parse::<SpacedRune>().unwrap_err().to_string(),
      "invalid character",
    );

    case("A.B", "AB", 0b1);
    case("A.B.C", "ABC", 0b11);
    case("A•B", "AB", 0b1);
    case("A•B•C", "ABC", 0b11);
  }

  #[test]
  fn serde() {
    let spaced_rune = SpacedRune {
      rune: Rune(26),
      spacers: 1,
    };
    let json = "\"A•A\"";
    assert_eq!(serde_json::to_string(&spaced_rune).unwrap(), json);
    assert_eq!(
      serde_json::from_str::<SpacedRune>(json).unwrap(),
      spaced_rune
    );
  }
}
