use super::*;

// This is for etching
#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Mint {
  pub deadline: Option<u32>, // unix timestamp
  pub limit: Option<u128>,   // max amount mintable per tx
  pub term: Option<u32>,     // relative to mint height
}
