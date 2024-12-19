use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Cenotaph {
  pub etching: Option<Rune>,
  pub flaw: Option<Flaw>,
  pub mint: Option<RuneId>,
}
