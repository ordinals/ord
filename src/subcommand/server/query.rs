use super::*;

pub(super) enum Block {
  Height(u32),
  Hash(BlockHash),
}

impl FromStr for Block {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if s.len() == 64 {
      Self::Hash(s.parse()?)
    } else {
      Self::Height(s.parse()?)
    })
  }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Inscription {
  Id(InscriptionId),
  Number(i32),
  Sat(Sat),
}

impl FromStr for Inscription {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if s.contains('i') {
      Self::Id(s.parse()?)
    } else if s.chars().all(|c| c.is_ascii_lowercase()) {
      Self::Sat(s.parse()?)
    } else {
      Self::Number(s.parse()?)
    })
  }
}

impl Display for Inscription {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Id(id) => write!(f, "{id}"),
      Self::Number(number) => write!(f, "{number}"),
      Self::Sat(sat) => write!(f, "on sat {}", sat.name()),
    }
  }
}

pub(super) enum Rune {
  SpacedRune(SpacedRune),
  RuneId(RuneId),
}

impl FromStr for Rune {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.contains(':') {
      Ok(Self::RuneId(s.parse()?))
    } else {
      Ok(Self::SpacedRune(s.parse()?))
    }
  }
}
