pub mod table;

use super::{LowerTick, ScriptKey, Tick};
use crate::inscriptions::InscriptionId;

fn script_tick_id_key(script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> String {
  format!(
    "{}_{}_{}",
    script,
    tick.to_lowercase().hex(),
    inscription_id
  )
}

fn min_script_tick_id_key(script: &ScriptKey, tick: &Tick) -> String {
  script_tick_key(script, tick)
}

fn max_script_tick_id_key(script: &ScriptKey, tick: &Tick) -> String {
  // because hex format of `InscriptionId` will be 0~f, so `g` is greater than `InscriptionId.to_string()` in bytes order
  format!("{}_{}_g", script, tick.to_lowercase().hex())
}

fn script_tick_key(script: &ScriptKey, tick: &Tick) -> String {
  format!("{}_{}", script, tick.to_lowercase().hex())
}

fn min_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, LowerTick::min_hex())
}

fn max_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, LowerTick::max_hex())
}
