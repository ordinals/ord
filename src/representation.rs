use {super::*, regex::RegexSet};

#[derive(Debug, Copy, Clone)]
pub(crate) enum Representation {
  CardinalAddress,
  Decimal,
  Degree,
  Hash,
  Integer,
  Name,
  OrdinalAddress,
  OutPoint,
  Percentile,
  SatPoint,
}

impl Representation {
  const fn pattern(self) -> (Self, &'static str) {
    (
      self,
      match self {
        Self::CardinalAddress => r"^(bc|BC|tb|TB|bcrt|BCRT)1.*$",
        Self::Decimal => r"^.*\..*$",
        Self::Degree => r"^.*°.*′.*″(.*‴)?$",
        Self::Hash => r"^[[:xdigit:]]{64}$",
        Self::Integer => r"^[0-9]*$",
        Self::Name => r"^[a-z]{1,11}$",
        Self::OrdinalAddress => OrdinalAddress::PATTERN,
        Self::OutPoint => r"^[[:xdigit:]]{64}:\d+$",
        Self::Percentile => r"^.*%$",
        Self::SatPoint => r"^[[:xdigit:]]{64}:\d+:\d+$",
      },
    )
  }
}

impl FromStr for Representation {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    if let Some(i) = REGEX_SET.matches(s).into_iter().next() {
      Ok(PATTERNS[i].0)
    } else {
      Err(anyhow!("unrecognized object"))
    }
  }
}

const PATTERNS: &[(Representation, &str)] = &[
  Representation::CardinalAddress.pattern(),
  Representation::Decimal.pattern(),
  Representation::Degree.pattern(),
  Representation::Hash.pattern(),
  Representation::Integer.pattern(),
  Representation::Name.pattern(),
  Representation::OrdinalAddress.pattern(),
  Representation::OutPoint.pattern(),
  Representation::Percentile.pattern(),
  Representation::SatPoint.pattern(),
];

lazy_static! {
  static ref REGEX_SET: RegexSet =
    RegexSet::new(PATTERNS.iter().map(|(_representation, pattern)| pattern),).unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_patterns_are_anchored() {
    assert!(PATTERNS
      .iter()
      .all(|(_representation, pattern)| pattern.starts_with('^') && pattern.ends_with('$')));
  }
}
