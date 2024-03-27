use super::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Terms {
  pub cap: u128,
  pub height: Option<Range>,
  pub limit: Decimal,
  pub offset: Option<Range>,
}
