use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Etching {
  pub divisibility: Option<u8>,
  pub premine: Option<u128>,
  pub rune: Option<Rune>,
  pub spacers: Option<u32>,
  pub symbol: Option<char>,
  pub terms: Option<Terms>,
}

impl Etching {
  pub const MAX_DIVISIBILITY: u8 = 38;
  pub const MAX_SPACERS: u32 = 0b00000111_11111111_11111111_11111111;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn max_spacers() {
    let mut rune = String::new();

    for (i, c) in Rune(u128::MAX).to_string().chars().enumerate() {
      if i > 0 {
        rune.push('•');
      }

      rune.push(c);
    }

    assert_eq!(
      Etching::MAX_SPACERS,
      rune.parse::<SpacedRune>().unwrap().spacers
    );
  }
}
