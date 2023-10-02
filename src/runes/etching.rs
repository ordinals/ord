use super::*;

#[derive(Default, Serialize, Debug, PartialEq)]
pub struct Etching {
  pub(crate) decimals: u128,
  pub(crate) rune: Rune,
}
