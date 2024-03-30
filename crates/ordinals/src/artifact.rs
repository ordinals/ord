use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub enum Artifact {
  Cenotaph(Cenotaph),
  Runestone(Runestone),
}

impl Artifact {
  pub fn mint(&self) -> Option<RuneId> {
    match self {
      Self::Cenotaph(cenotaph) => cenotaph.mint,
      Self::Runestone(runestone) => runestone.mint,
    }
  }

  pub fn etching(&self) -> Option<Rune> {
    match self {
      Self::Cenotaph(cenotaph) => cenotaph.etching,
      Self::Runestone(runestone) => runestone.etching.and_then(|etching| etching.rune),
    }
  }
}
