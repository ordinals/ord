use super::*;

#[derive(Debug, PartialEq, Default)]
pub struct Cenotaph {
  pub flaws: u32,
  pub mint: Option<RuneId>,
  pub etching: Option<Rune>,
}
