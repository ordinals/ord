use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Outgoing {
  Amount(Amount),
  InscriptionId(InscriptionId),
  SatPoint(SatPoint),
  Rune { decimal: Decimal, rune: SpacedRune },
}

impl Display for Outgoing {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Amount(amount) => amount.fmt(f),
      Self::InscriptionId(inscription_id) => inscription_id.fmt(f),
      Self::SatPoint(satpoint) => satpoint.fmt(f),
      Self::Rune { decimal, rune } => write!(f, "{decimal} {rune}"),
    }
  }
}

impl FromStr for Outgoing {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    lazy_static! {
      static ref SATPOINT: Regex = Regex::new(r"^[[:xdigit:]]{64}:\d+:\d+$").unwrap();
      static ref INSCRIPTION_ID: Regex = Regex::new(r"^[[:xdigit:]]{64}i\d+$").unwrap();
      static ref AMOUNT: Regex = Regex::new(
        r"(?x)
        ^
        (
          \d+
          |
          \.\d+
          |
          \d+\.\d+
        )
        \ *
        (bit|btc|cbtc|mbtc|msat|nbtc|pbtc|sat|satoshi|ubtc)
        (s)?
        $
        "
      )
      .unwrap();
      static ref RUNE: Regex = Regex::new(
        r"(?x)
        ^
        (
          \d+
          |
          \.\d+
          |
          \d+\.\d+
        )
        \ *
        (
          [A-Z•.]+
        )
        $
        "
      )
      .unwrap();
    }

    Ok(if SATPOINT.is_match(s) {
      Self::SatPoint(s.parse()?)
    } else if INSCRIPTION_ID.is_match(s) {
      Self::InscriptionId(s.parse()?)
    } else if AMOUNT.is_match(s) {
      Self::Amount(s.parse()?)
    } else if let Some(captures) = RUNE.captures(s) {
      Self::Rune {
        decimal: captures[1].parse()?,
        rune: captures[2].parse()?,
      }
    } else {
      bail!("unrecognized outgoing: {s}");
    })
  }
}

impl Serialize for Outgoing {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for Outgoing {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, outgoing: Outgoing) {
      assert_eq!(s.parse::<Outgoing>().unwrap(), outgoing);
    }

    case(
      "0000000000000000000000000000000000000000000000000000000000000000i0",
      Outgoing::InscriptionId(
        "0000000000000000000000000000000000000000000000000000000000000000i0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "0000000000000000000000000000000000000000000000000000000000000000:0:0",
      Outgoing::SatPoint(
        "0000000000000000000000000000000000000000000000000000000000000000:0:0"
          .parse()
          .unwrap(),
      ),
    );

    case("0 btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case("0btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case("0.0btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case(".0btc", Outgoing::Amount("0 btc".parse().unwrap()));

    case(
      "0 XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "0XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "0.0XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "0.0".parse().unwrap(),
      },
    );

    case(
      ".0XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: ".0".parse().unwrap(),
      },
    );

    case(
      "1.1XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );

    case(
      "1.1X.Y.Z",
      Outgoing::Rune {
        rune: "X.Y.Z".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );

    assert!("0".parse::<Outgoing>().is_err());
  }
}
