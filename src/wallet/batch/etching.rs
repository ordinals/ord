use super::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Etching {
  pub divisibility: u8,
  pub mint: Option<Mint>,
  pub premine: Decimal,
  pub rune: SpacedRune,
  pub supply: Decimal,
  pub symbol: char,
}
