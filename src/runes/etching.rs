use super::*;

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Etching {
  pub divisibility: u8,
  pub mint: Option<Mint>,
  pub rune: Option<Rune>,
  pub spacers: u32,
  pub symbol: Option<char>,
}
