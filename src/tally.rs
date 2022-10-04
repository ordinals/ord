use super::*;

pub(crate) trait Tally {
  fn tally(self, count: usize) -> Tallied;
}

impl Tally for &'static str {
  fn tally(self, count: usize) -> Tallied {
    Tallied { noun: self, count }
  }
}

pub(crate) struct Tallied {
  count: usize,
  noun: &'static str,
}

impl Display for Tallied {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.count == 1 {
      write!(f, "{} {}", self.count, self.noun)
    } else {
      write!(f, "{} {}s", self.count, self.noun)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn zero() {
    assert_eq!("foo".tally(0).to_string(), "0 foos")
  }

  #[test]
  fn one() {
    assert_eq!("foo".tally(1).to_string(), "1 foo")
  }

  #[test]
  fn two() {
    assert_eq!("foo".tally(2).to_string(), "2 foos")
  }
}
