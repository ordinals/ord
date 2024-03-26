use super::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Mint {
  pub deadline: Option<u32>,
  pub limit: Decimal,
  pub cap: u128,
  pub term: Option<u32>,
}
