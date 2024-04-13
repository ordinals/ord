use super::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Range {
  pub start: Option<u64>,
  pub end: Option<u64>,
}
