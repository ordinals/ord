use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct FeeRate(u64);

impl FromStr for FeeRate {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let fee_rate = f64::from_str(s)?;

    if fee_rate.is_sign_negative() | fee_rate.is_nan() | fee_rate.is_infinite() {
      return Err(anyhow!("fee rate can not be negative"));
    }

    Self::try_from(fee_rate)
  }
}

impl TryFrom<f64> for FeeRate {
  type Error = Error;

  fn try_from(float: f64) -> Result<Self, Self::Error> {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    Ok(Self((float * 1000.0).round() as u64))
  }
}

impl FeeRate {
  pub(crate) fn fee(&self, vbytes: usize) -> Amount {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    Amount::from_sat((self.0 as f64 * vbytes as f64 / 1000.0).ceil() as u64)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    assert_eq!("1.0".parse::<FeeRate>().unwrap().0, 1000);
    assert_eq!("11.1119".parse::<FeeRate>().unwrap().0, 11112);
    assert_eq!("11.1111".parse::<FeeRate>().unwrap().0, 11111);
    assert!("-4.2".parse::<FeeRate>().is_err());
  }

  #[test]
  fn fee() {
    assert_eq!(
      "2.5".parse::<FeeRate>().unwrap().fee(100),
      Amount::from_sat(250)
    );
    assert_eq!(
      "2.0".parse::<FeeRate>().unwrap().fee(1024),
      Amount::from_sat(2048)
    );
    assert_eq!(
      "1.1".parse::<FeeRate>().unwrap().fee(1),
      Amount::from_sat(2)
    );
    assert_eq!(
      "1.0".parse::<FeeRate>().unwrap().fee(123456789),
      Amount::from_sat(123456789)
    );
  }
}
