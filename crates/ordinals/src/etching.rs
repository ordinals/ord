use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Etching {
  pub divisibility: Option<u8>,
  pub premine: Option<u128>,
  pub rune: Option<Rune>,
  pub spacers: Option<u32>,
  pub symbol: Option<char>,
  pub terms: Option<Terms>,
  pub turbo: bool,
}

impl Etching {
  pub const MAX_DIVISIBILITY: u8 = 38;
  pub const MAX_SPACERS: u32 = 0b00000111_11111111_11111111_11111111;

  pub fn supply(&self) -> Option<u128> {
    let premine = self.premine.unwrap_or_default();
    let cap = self.terms.and_then(|terms| terms.cap).unwrap_or_default();
    let amount = self
      .terms
      .and_then(|terms| terms.amount)
      .unwrap_or_default();
    premine.checked_add(cap.checked_mul(amount)?)
  }
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

  #[test]
  fn supply() {
    #[track_caller]
    fn case(premine: Option<u128>, terms: Option<Terms>, supply: Option<u128>) {
      assert_eq!(
        Etching {
          premine,
          terms,
          ..default()
        }
        .supply(),
        supply,
      );
    }

    case(None, None, Some(0));
    case(Some(0), None, Some(0));
    case(Some(1), None, Some(1));
    case(
      Some(1),
      Some(Terms {
        cap: None,
        amount: None,
        ..default()
      }),
      Some(1),
    );

    case(
      None,
      Some(Terms {
        cap: None,
        amount: None,
        ..default()
      }),
      Some(0),
    );

    case(
      Some(u128::MAX / 2 + 1),
      Some(Terms {
        cap: Some(u128::MAX / 2),
        amount: Some(1),
        ..default()
      }),
      Some(u128::MAX),
    );

    case(
      Some(1000),
      Some(Terms {
        cap: Some(10),
        amount: Some(100),
        ..default()
      }),
      Some(2000),
    );

    case(
      Some(u128::MAX),
      Some(Terms {
        cap: Some(1),
        amount: Some(1),
        ..default()
      }),
      None,
    );

    case(
      Some(0),
      Some(Terms {
        cap: Some(1),
        amount: Some(u128::MAX),
        ..default()
      }),
      Some(u128::MAX),
    );
  }
}
