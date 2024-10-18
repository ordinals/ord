use super::*;

#[derive(Debug, PartialEq, Clone, DeserializeFromStr, SerializeDisplay)]
pub enum Outgoing {
  Amount(Amount),
  InscriptionId(InscriptionId),
  Rune { decimal: Decimal, rune: SpacedRune },
  Sat(Sat),
  SatPoint(SatPoint),
}

impl Display for Outgoing {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Amount(amount) => write!(f, "{}", amount.to_string().to_lowercase()),
      Self::InscriptionId(inscription_id) => inscription_id.fmt(f),
      Self::Rune { decimal, rune } => write!(f, "{decimal}:{rune}"),
      Self::Sat(sat) => write!(f, "{}", sat.name()),
      Self::SatPoint(satpoint) => satpoint.fmt(f),
    }
  }
}

impl FromStr for Outgoing {
  type Err = SnafuError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    lazy_static! {
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
        \ ?
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
        \s*:\s*
        (
          [A-Z•.]+
        )
        $
        "
      )
      .unwrap();
    }

    if re::SAT_NAME.is_match(input) {
      Ok(Outgoing::Sat(
        input.parse().snafu_context(error::SatParse { input })?,
      ))
    } else if re::SATPOINT.is_match(input) {
      Ok(Outgoing::SatPoint(
        input
          .parse()
          .snafu_context(error::SatPointParse { input })?,
      ))
    } else if re::INSCRIPTION_ID.is_match(input) {
      Ok(Outgoing::InscriptionId(
        input
          .parse()
          .snafu_context(error::InscriptionIdParse { input })?,
      ))
    } else if AMOUNT.is_match(input) {
      Ok(Outgoing::Amount(
        input.parse().snafu_context(error::AmountParse { input })?,
      ))
    } else if let Some(captures) = RUNE.captures(input) {
      let decimal = captures[1]
        .parse::<Decimal>()
        .snafu_context(error::RuneAmountParse { input })?;
      let rune = captures[2]
        .parse()
        .snafu_context(error::RuneParse { input })?;
      Ok(Self::Rune { decimal, rune })
    } else {
      Err(SnafuError::OutgoingParse {
        input: input.to_string(),
      })
    }
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

    case("nvtdijuwxlp", Outgoing::Sat("nvtdijuwxlp".parse().unwrap()));
    case("a", Outgoing::Sat("a".parse().unwrap()));

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
      "0  : XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "0:XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "0.0:XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "0.0".parse().unwrap(),
      },
    );

    case(
      ".0:XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: ".0".parse().unwrap(),
      },
    );

    case(
      "1.1:XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );

    case(
      "1.1:X.Y.Z",
      Outgoing::Rune {
        rune: "X.Y.Z".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );
  }

  #[test]
  fn roundtrip() {
    #[track_caller]
    fn case(s: &str, outgoing: Outgoing) {
      assert_eq!(s.parse::<Outgoing>().unwrap(), outgoing);
      assert_eq!(s, outgoing.to_string());
    }

    case("nvtdijuwxlp", Outgoing::Sat("nvtdijuwxlp".parse().unwrap()));
    case("a", Outgoing::Sat("a".parse().unwrap()));

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
    case(
      "1.20000000 btc",
      Outgoing::Amount("1.2 btc".parse().unwrap()),
    );

    case(
      "0:XY•Z",
      Outgoing::Rune {
        rune: "XY•Z".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "1.1:XYZ",
      Outgoing::Rune {
        rune: "XYZ".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );
  }

  #[test]
  fn serde() {
    #[track_caller]
    fn case(s: &str, j: &str, o: Outgoing) {
      assert_eq!(s.parse::<Outgoing>().unwrap(), o);
      assert_eq!(serde_json::to_string(&o).unwrap(), j);
      assert_eq!(serde_json::from_str::<Outgoing>(j).unwrap(), o);
    }

    case(
      "nvtdijuwxlp",
      "\"nvtdijuwxlp\"",
      Outgoing::Sat("nvtdijuwxlp".parse().unwrap()),
    );
    case("a", "\"a\"", Outgoing::Sat("a".parse().unwrap()));

    case(
      "0000000000000000000000000000000000000000000000000000000000000000i0",
      "\"0000000000000000000000000000000000000000000000000000000000000000i0\"",
      Outgoing::InscriptionId(
        "0000000000000000000000000000000000000000000000000000000000000000i0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "0000000000000000000000000000000000000000000000000000000000000000:0:0",
      "\"0000000000000000000000000000000000000000000000000000000000000000:0:0\"",
      Outgoing::SatPoint(
        "0000000000000000000000000000000000000000000000000000000000000000:0:0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "3 btc",
      "\"3 btc\"",
      Outgoing::Amount(Amount::from_sat(3 * COIN_VALUE)),
    );

    case(
      "6.66:HELL.MONEY",
      "\"6.66:HELL•MONEY\"",
      Outgoing::Rune {
        rune: "HELL•MONEY".parse().unwrap(),
        decimal: "6.66".parse().unwrap(),
      },
    );
  }
}
