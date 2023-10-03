use super::*;

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone, PartialOrd)]
pub(crate) struct Rune(pub(crate) u128);

impl Rune {
  pub(crate) fn minimum_at_height(height: Height) -> Self {
    Self(u128::from(
      Sat::SUPPLY - height.starting_sat().0 - height.subsidy(),
    ))
  }
}

impl Display for Rune {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut n = self.0;
    if n == u128::max_value() {
      return write!(f, "BCGDENLQRQWDSLRUGSNLBTMFIJAV");
    }

    n += 1;
    let mut symbol = String::new();
    while n > 0 {
      symbol.push(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
          .chars()
          .nth(((n - 1) % 26) as usize)
          .unwrap(),
      );
      n = (n - 1) / 26;
    }

    for c in symbol.chars().rev() {
      write!(f, "{c}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_string() {
    assert_eq!(Rune(0).to_string(), "A");
    assert_eq!(Rune(1).to_string(), "B");
    assert_eq!(Rune(2).to_string(), "C");
    assert_eq!(Rune(3).to_string(), "D");
    assert_eq!(Rune(4).to_string(), "E");
    assert_eq!(Rune(5).to_string(), "F");
    assert_eq!(Rune(6).to_string(), "G");
    assert_eq!(Rune(7).to_string(), "H");
    assert_eq!(Rune(8).to_string(), "I");
    assert_eq!(Rune(9).to_string(), "J");
    assert_eq!(Rune(10).to_string(), "K");
    assert_eq!(Rune(11).to_string(), "L");
    assert_eq!(Rune(12).to_string(), "M");
    assert_eq!(Rune(13).to_string(), "N");
    assert_eq!(Rune(14).to_string(), "O");
    assert_eq!(Rune(15).to_string(), "P");
    assert_eq!(Rune(16).to_string(), "Q");
    assert_eq!(Rune(17).to_string(), "R");
    assert_eq!(Rune(18).to_string(), "S");
    assert_eq!(Rune(19).to_string(), "T");
    assert_eq!(Rune(20).to_string(), "U");
    assert_eq!(Rune(21).to_string(), "V");
    assert_eq!(Rune(22).to_string(), "W");
    assert_eq!(Rune(23).to_string(), "X");
    assert_eq!(Rune(24).to_string(), "Y");
    assert_eq!(Rune(25).to_string(), "Z");
    assert_eq!(Rune(26).to_string(), "AA");
    assert_eq!(Rune(27).to_string(), "AB");
    assert_eq!(Rune(51).to_string(), "AZ");
    assert_eq!(Rune(52).to_string(), "BA");
    assert_eq!(
      Rune(u128::max_value() - 1).to_string(),
      "BCGDENLQRQWDSLRUGSNLBTMFIJAU"
    );
    assert_eq!(
      Rune(u128::max_value()).to_string(),
      "BCGDENLQRQWDSLRUGSNLBTMFIJAV"
    );
  }

  #[test]
  fn minimum_for_height() {
    assert_eq!(Rune::minimum_at_height(Sat::LAST.height()).to_string(), "A");
    assert_eq!(
      Rune::minimum_at_height(Height(0)).to_string(),
      Sat(50 * COIN_VALUE - 1).name().to_uppercase()
    );
    assert_eq!(
      Rune::minimum_at_height(Height(1)).to_string(),
      Sat(100 * COIN_VALUE - 1).name().to_uppercase()
    );
  }
}
