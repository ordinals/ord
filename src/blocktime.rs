use super::*;

#[derive(Copy, Clone)]
pub(crate) enum Blocktime {
  Confirmed(i64),
  Expected(i64),
}

impl Blocktime {
  fn timestamp(self) -> i64 {
    match self {
      Self::Confirmed(timestamp) | Self::Expected(timestamp) => timestamp,
    }
  }
}

impl Display for Blocktime {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      NaiveDateTime::from_timestamp_opt(self.timestamp(), 0).unwrap()
    )?;

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
    assert_eq!(Blocktime::Confirmed(0).to_string(), "1970-01-01 00:00:00");
    assert_eq!(
      Blocktime::Expected(0).to_string(),
      "1970-01-01 00:00:00 (expected)"
    );
  }
}
