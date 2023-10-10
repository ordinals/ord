use super::*;

pub(crate) struct Pile {
  pub(crate) amount: u128,
  pub(crate) divisibility: u8,
}

impl Display for Pile {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let x = 10u128.pow(self.divisibility.into());

    let whole = self.amount / x;
    let fractional = self.amount % x;

    if fractional > 0 {
      write!(
        f,
        "{whole}.{fractional:0>width$}",
        width = self.divisibility.into()
      )
    } else {
      write!(f, "{whole}")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      Pile {
        amount: 0,
        divisibility: 0
      }
      .to_string(),
      "0"
    );
    assert_eq!(
      Pile {
        amount: 25,
        divisibility: 0
      }
      .to_string(),
      "25"
    );
    assert_eq!(
      Pile {
        amount: 0,
        divisibility: 1,
      }
      .to_string(),
      "0"
    );
    assert_eq!(
      Pile {
        amount: 1,
        divisibility: 1,
      }
      .to_string(),
      "0.1"
    );
    assert_eq!(
      Pile {
        amount: 1,
        divisibility: 2,
      }
      .to_string(),
      "0.01"
    );
    assert_eq!(
      Pile {
        amount: 10,
        divisibility: 2,
      }
      .to_string(),
      "0.10"
    );
    assert_eq!(
      Pile {
        amount: 100,
        divisibility: 2,
      }
      .to_string(),
      "1"
    );
    assert_eq!(
      Pile {
        amount: 101,
        divisibility: 2,
      }
      .to_string(),
      "1.01"
    );
    assert_eq!(
      Pile {
        amount: u128::max_value(),
        divisibility: 18,
      }
      .to_string(),
      "340282366920938463463.374607431768211455"
    );
  }
}
