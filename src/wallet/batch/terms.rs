use super::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Terms {
  pub amount: Decimal,
  pub base: Option<RuneId>,
  pub cap: u128,
  pub height: Option<Range>,
  pub offset: Option<Range>,
  pub price: Option<u128>,
  pub seed: Option<u128>,
}
