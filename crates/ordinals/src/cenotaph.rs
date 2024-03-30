use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Cenotaph {
  pub flaws: u32,
  pub mint: Option<RuneId>,
  pub etching: Option<Rune>,
}

impl Cenotaph {
  pub fn flaws(&self) -> Vec<Flaw> {
    Flaw::ALL
      .into_iter()
      .filter(|flaw| self.flaws & flaw.flag() != 0)
      .collect()
  }
}
