use super::*;

#[derive(Default, Serialize, Debug, PartialEq, Clone)]
pub struct Edict {
  pub id: u128,
  pub amount: u128,
  pub output: u128,
}
