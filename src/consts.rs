use super::*;

pub(crate) const SUPPLY: u64 = 2099999997690000;

pub(crate) const INITIAL_SUBSIDY: u64 = 50 * COIN_VALUE;

#[cfg(test)]
mod tests {
  use super::super::*;

  #[test]
  fn supply() {
    let mut mined = 0;

    for height in 0.. {
      let subsidy = Height(height).subsidy();

      if subsidy == 0 {
        break;
      }

      mined += subsidy;
    }

    assert_eq!(SUPPLY, mined);
  }
}
