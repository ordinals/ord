use super::*;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Terms {
  pub amount: Decimal,
  pub cap: u128,
  pub height: Option<Range>,
  pub offset: Option<Range>,
}
