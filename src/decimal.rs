use super::*;

#[derive(PartialEq, Debug)]
pub(crate) struct Decimal {
  height: Height,
  offset: u64,
}

impl From<Ordinal> for Decimal {
  fn from(ordinal: Ordinal) -> Self {
    Self {
      height: ordinal.height(),
      offset: ordinal.third(),
    }
  }
}

impl Display for Decimal {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}.{}", self.height, self.offset)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decimal() {
    assert_eq!(
      Ordinal(0).decimal(),
      Decimal {
        height: Height(0),
        offset: 0
      }
    );
    assert_eq!(
      Ordinal(1).decimal(),
      Decimal {
        height: Height(0),
        offset: 1
      }
    );
    assert_eq!(
      Ordinal(2099999997689999).decimal(),
      Decimal {
        height: Height(6929999),
        offset: 0
      }
    );
  }
}
