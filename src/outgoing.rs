use {
  super::*,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
};

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outgoing {
  #[serde(deserialize_with = "deserialize_amount")]
  Amount(Amount),
  InscriptionId(InscriptionId),
  SatPoint(SatPoint),
  #[serde(deserialize_with = "deserialize_rune")]
  Rune {
    decimal: Decimal,
    rune: SpacedRune,
  },
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
    match *self {
      Outgoing::Amount(ref amount) => {
        serializer.serialize_newtype_variant("Outgoing", 0, "amount", &amount.to_string())
      }
      Outgoing::InscriptionId(ref id) => {
        serializer.serialize_newtype_variant("Outgoing", 1, "inscription_id", &id.to_string())
      }
      Outgoing::SatPoint(ref satpoint) => {
        serializer.serialize_newtype_variant("Outgoing", 2, "sat_point", &satpoint.to_string())
      }
      Outgoing::Rune { decimal, rune } => serializer.serialize_newtype_variant(
        "Outgoing",
        3,
        "rune",
        &format!("{} {}", decimal, rune),
      ),
    }
  }
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  Amount::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_rune<'de, D>(deserializer: D) -> Result<(Decimal, SpacedRune), D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  let (decimal, rune) = s.split_once(" ").unwrap_or_default();

  Ok((
    Decimal::from_str(&decimal).map_err(serde::de::Error::custom)?,
    SpacedRune::from_str(&rune).map_err(serde::de::Error::custom)?,
  ))
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
