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
