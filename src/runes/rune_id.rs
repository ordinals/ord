use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct RuneId {
  pub(crate) height: u64,
  pub(crate) index: u16,
}

impl From<RuneId> for u128 {
  fn from(id: RuneId) -> Self {
    u128::from(id.height) << 16 | u128::from(id.index)
  }
}
