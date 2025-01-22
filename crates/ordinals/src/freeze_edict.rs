use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct FreezeEdict {
  pub rune_id: Option<RuneId>,
  pub outpoints: Vec<OutpointId>,
}
