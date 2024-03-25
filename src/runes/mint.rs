use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Mint {
  pub cap: Option<u128>,     // mint cap
  pub deadline: Option<u32>, // unix timestamp
  pub limit: Option<u128>,   // claim amount
  pub term: Option<u32>,     // relative block height
}
