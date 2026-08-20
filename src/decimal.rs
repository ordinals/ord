use super::*;

#[derive(Debug, PartialEq, Copy, Clone, Default, DeserializeFromStr, SerializeDisplay)]
pub struct Decimal {
  pub value: u128,
  pub scale: u8,
}

impl Decimal {
  pub fn to_integer(self, divisibility: u8) -> Result<u128> {
    match divisibility.checked_sub(self.scale) {
      Some(difference) => Ok(
        self
          .value
          .checked_mul(
            10u128
              .checked_pow(u32::from(difference))
              .context("divisibility out of range")?,
          )
          .context("amount out of range")?,
      ),
      None => bail!("excessive precision"),
    }
  }
}

impl Display for Decimal {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let magnitude = 10u128.checked_pow(self.scale.into()).ok_or(fmt::Error)?;

    let integer = self.value / magnitude;
    let mut fraction = self.value % magnitude;

    write!(f, "{integer}")?;

    if fraction > 0 {
      let mut width = self.scale.into();

      while fraction.is_multiple_of(10) {
        fraction /= 10;
        width -= 1;
      }

      write!(f, ".{fraction:0>width$}")?;
    }

    Ok(())
  }
}

impl FromStr for Decimal {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Some((integer, decimal)) = s.split_once('.') {
      if integer.is_empty() && decimal.is_empty() {
        bail!("empty decimal");
      }

      let integer = if integer.is_empty() {
        0
      } else {
        integer.parse::<u128>()?
      };

      let (decimal, scale) = if decimal.is_empty() {
        (0, 0)
      } else {
        let trailing_zeros = decimal.chars().rev().take_while(|c| *c == '0').count();
        let significant_digits = decimal.chars().count() - trailing_zeros;
        let decimal = decimal.parse::<u128>()?
          / 10u128
            .checked_pow(u32::try_from(trailing_zeros).unwrap())
            .context("excessive trailing zeros")?;
        (
          decimal,
          u8::try_from(significant_digits).context("excessive precision")?,
        )
      };

      Ok(Self {
        value: integer
          .checked_mul(
            10u128
              .checked_pow(u32::from(scale))
              .context("excessive precision")?,
          )
          .context("integer overflows maximum value")?
          .checked_add(decimal)
          .context("decimal overflows maximum value")?,
        scale,
      })
    } else {
      Ok(Self {
        value: s.parse::<u128>()?,
        scale: 0,
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
    fn ok(s: &str, value: u128, scale: u8) {
      assert_eq!(s.parse::<Decimal>().unwrap(), Decimal { value, scale });
    }

    #[track_caller]
    fn err(s: &str, expected: &str) {
      assert_eq!(s.parse::<Decimal>().unwrap_err().to_string(), expected);
    }

    err(".", "empty decimal");

    err("a.b", "invalid digit found in string");

    err(" 0.1 ", "invalid digit found in string");

    err(
      &format!("{}.1", u128::MAX),
      "integer overflows maximum value",
    );

    err(
      "0.000000000000000000000000000000000000001",
      "excessive precision",
    );

    err(&format!("0.{}1", "0".repeat(255)), "excessive precision");

    ok("0", 0, 0);
    ok("0.00000", 0, 0);
    ok("1.0", 1, 0);
    ok("1.1", 11, 1);
    ok("1.11", 111, 2);
    ok("1.", 1, 0);
    ok(".1", 1, 1);
    ok("1.10", 11, 1);
  }

  #[test]
  fn to_amount() {
    #[track_caller]
    fn case(s: &str, divisibility: u8, amount: u128) {
      assert_eq!(
        s.parse::<Decimal>()
          .unwrap()
          .to_integer(divisibility)
          .unwrap(),
        amount,
      );
    }

    assert_eq!(
      Decimal { value: 0, scale: 0 }
        .to_integer(255)
        .unwrap_err()
        .to_string(),
      "divisibility out of range"
    );

    assert_eq!(
      Decimal {
        value: u128::MAX,
        scale: 0,
      }
      .to_integer(1)
      .unwrap_err()
      .to_string(),
      "amount out of range",
    );

    assert_eq!(
      Decimal { value: 1, scale: 1 }
        .to_integer(0)
        .unwrap_err()
        .to_string(),
      "excessive precision",
    );

    case("1", 0, 1);
    case("1.0", 0, 1);
    case("1.0", 1, 10);
    case("1.2", 1, 12);
    case("1.2", 2, 120);
    case("123.456", 3, 123456);
    case("123.456", 6, 123456000);
  }

  #[test]
  fn to_string() {
    #[track_caller]
    fn case(decimal: Decimal, string: &str) {
      assert_eq!(decimal.to_string(), string);
      assert_eq!(decimal, string.parse::<Decimal>().unwrap());
    }

    case(Decimal { value: 0, scale: 0 }, "0");
    case(Decimal { value: 1, scale: 0 }, "1");
    case(Decimal { value: 1, scale: 1 }, "0.1");
    case(
      Decimal {
        value: 101,
        scale: 2,
      },
      "1.01",
    );
    case(
      Decimal {
        value: 1234,
        scale: 6,
      },
      "0.001234",
    );
    case(
      Decimal {
        value: 12,
        scale: 0,
      },
      "12",
    );
    case(
      Decimal {
        value: 12,
        scale: 1,
      },
      "1.2",
    );
    case(
      Decimal {
        value: 12,
        scale: 2,
      },
      "0.12",
    );
    case(
      Decimal {
        value: 123456,
        scale: 3,
      },
      "123.456",
    );
    case(
      Decimal {
        value: 123456789,
        scale: 6,
      },
      "123.456789",
    );
  }
}
