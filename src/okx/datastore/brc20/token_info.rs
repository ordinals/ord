use super::*;
use crate::InscriptionId;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TokenInfo {
  pub tick: Tick,
  pub inscription_id: InscriptionId,
  pub inscription_number: i32,
  pub supply: u128,
  pub minted: u128,
  pub limit_per_mint: u128,
  pub decimal: u8,
  pub deploy_by: ScriptKey,
  pub deployed_number: u32,
  pub deployed_timestamp: u32,
  pub latest_mint_number: u32,
}
