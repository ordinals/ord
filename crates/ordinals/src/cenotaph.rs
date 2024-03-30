use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Cenotaph {
  pub flaws: u32,
  pub mint: Option<RuneId>,
  pub etching: Option<Rune>,
}
