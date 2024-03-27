use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Terms {
  pub cap: Option<u128>,
  pub height: (Option<u64>, Option<u64>),
  pub limit: Option<u128>,
  pub offset: (Option<u64>, Option<u64>),
}
