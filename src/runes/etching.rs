use super::*;

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Etching {
  pub divisibility: u8,
  pub rune: Option<Rune>,
  pub spacers: u32,
  pub symbol: Option<char>,
  pub deadline: Option<u32>,
  pub limit: Option<u128>,
  pub term: Option<u32>,
}
