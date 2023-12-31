use super::*;
use crate::InscriptionId;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransferableLog {
  pub inscription_id: InscriptionId,
  pub inscription_number: i32,
  pub amount: u128,
  pub tick: Tick,
  pub owner: ScriptKey,
}
