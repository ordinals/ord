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
    if re::INSCRIPTION_ID.is_match(s) {
      Ok(Self::Id(s.parse()?))
    } else if re::INSCRIPTION_NUMBER.is_match(s) {
      Ok(Self::Number(s.parse()?))
    } else if re::SAT_NAME.is_match(s) {
      Ok(Self::Sat(s.parse()?))
    } else {
      Err(anyhow!("bad inscription query {s}"))
    }
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

#[derive(Debug)]
pub(super) enum Rune {
  Spaced(SpacedRune),
  Id(RuneId),
  Number(u64),
}

impl FromStr for Rune {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.contains(':') {
      Ok(Self::Id(s.parse()?))
    } else if re::RUNE_NUMBER.is_match(s) {
      Ok(Self::Number(s.parse()?))
    } else {
      Ok(Self::Spaced(s.parse()?))
    }
  }
}
