use super::*;

// todo: rename to terms, conditions, or provisos
#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Open {
  pub deadline: Option<u32>,
  pub limit: Option<u128>,
  pub term: Option<u32>,
}
