use super::*;

#[derive(Default, Serialize, Debug, PartialEq)]
pub struct Edict {
  pub id: u128,
  pub amount: u128,
  pub output: u128,
}
