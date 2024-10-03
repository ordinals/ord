use super::*;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Etching {
  pub rune: SpacedRune,
  pub symbol: char,
  pub divisibility: u8,
  pub supply: Decimal,
  pub premine: Decimal,
  pub terms: Option<batch::Terms>,
  pub turbo: bool,
}
