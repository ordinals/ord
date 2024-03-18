use super::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Etch {
  pub divisibility: u8,
  pub mint: Option<BatchMint>,
  pub premine: Decimal,
  pub rune: SpacedRune,
  pub symbol: char,
}
