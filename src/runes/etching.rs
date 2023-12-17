use super::*;

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Etching {
  pub deadline: Option<u32>,
  pub divisibility: u8,
  pub limit: Option<u128>,
  pub rune: Option<Rune>,
  pub symbol: Option<char>,
  pub term: Option<u32>,
  pub spacers: u32,
}
