use super::*;

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Mint {
  pub deadline: Option<u32>, // unix timestamp
  pub limit: Option<u128>,   // claim amount
  pub term: Option<u32>,     // relative block height
}
