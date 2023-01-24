use super::*;

#[derive(Copy, Clone)]
pub(crate) enum Blocktime {
  Confirmed(DateTime<Utc>),
  Expected(DateTime<Utc>),
}

impl Blocktime {
  pub(crate) fn confirmed(seconds: u32) -> Self {
    Self::Confirmed(timestamp(seconds))
  }

  pub(crate) fn expected(seconds: u32) -> Self {
    Self::Expected(timestamp(seconds))
  }

  fn timestamp(self) -> DateTime<Utc> {
    match self {
      Self::Confirmed(timestamp) | Self::Expected(timestamp) => timestamp,
    }
  }
}

impl Display for Blocktime {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.timestamp())?;

    if let Self::Expected(_) = self {
      write!(f, " (expected)")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      Blocktime::confirmed(0).to_string(),
      "1970-01-01 00:00:00 UTC"
    );
    assert_eq!(
      Blocktime::expected(0).to_string(),
      "1970-01-01 00:00:00 UTC (expected)"
    );
  }
}
