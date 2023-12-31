use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferInfo {
  pub tick: Tick,
  pub amt: u128,
}
