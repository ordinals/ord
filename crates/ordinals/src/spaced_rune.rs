use super::*;

#[derive(
  Copy, Clone, Debug, PartialEq, Ord, PartialOrd, Eq, Default, DeserializeFromStr, SerializeDisplay,
)]
pub struct SpacedRune {
  pub rune: Rune,
  pub spacers: u32,
}

impl SpacedRune {
  pub fn new(rune: Rune, spacers: u32) -> Self {
    Self { rune, spacers }
  }
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
          let flag = 1 << rune.len().checked_sub(1).ok_or(Error::LeadingSpacer)?;
          if spacers & flag != 0 {
            return Err(Error::DoubleSpacer);
          }
          spacers |= flag;
        }
        _ => return Err(Error::Character(c)),
      }
    }

    if 32 - spacers.leading_zeros() >= rune.len().try_into().unwrap() {
      return Err(Error::TrailingSpacer);
    }

    Ok(SpacedRune {
      rune: rune.parse().map_err(Error::Rune)?,
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

#[derive(Debug, PartialEq)]
pub enum Error {
  LeadingSpacer,
  TrailingSpacer,
  DoubleSpacer,
  Character(char),
  Rune(rune::Error),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Character(c) => write!(f, "invalid character `{c}`"),
      Self::DoubleSpacer => write!(f, "double spacer"),
      Self::LeadingSpacer => write!(f, "leading spacer"),
      Self::TrailingSpacer => write!(f, "trailing spacer"),
      Self::Rune(err) => write!(f, "{err}"),
    }
  }
}

impl std::error::Error for Error {}

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
      ".A".parse::<SpacedRune>().unwrap_err(),
      Error::LeadingSpacer,
    );

    assert_eq!(
      "A..B".parse::<SpacedRune>().unwrap_err(),
      Error::DoubleSpacer,
    );

    assert_eq!(
      "A.".parse::<SpacedRune>().unwrap_err(),
      Error::TrailingSpacer,
    );

    assert_eq!(
      "Ax".parse::<SpacedRune>().unwrap_err(),
      Error::Character('x')
    );

    case("A.B", "AB", 0b1);
    case("A.B.C", "ABC", 0b11);
    case("A•B", "AB", 0b1);
    case("A•B•C", "ABC", 0b11);
    case("A•BC", "ABC", 0b1);
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
