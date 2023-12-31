pub mod table;

use super::{LowerTick, ScriptKey, Tick};

fn script_tick_key(script: &ScriptKey, tick: &Tick) -> String {
  format!("{}_{}", script, tick.to_lowercase().hex())
}

fn min_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, LowerTick::min_hex())
}

fn max_script_tick_key(script: &ScriptKey) -> String {
  format!("{}_{}", script, LowerTick::max_hex())
}
