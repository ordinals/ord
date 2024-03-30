use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Cenotaph {
  pub etching: Option<Rune>,
  pub flaws: u32,
  pub mint: Option<RuneId>,
}

impl Cenotaph {
  pub fn flaws(&self) -> Vec<Flaw> {
    Flaw::ALL
      .into_iter()
      .filter(|flaw| self.flaws & flaw.flag() != 0)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn flaws() {
    assert_eq!(
      Cenotaph {
        flaws: Flaw::Opcode.flag() | Flaw::Varint.flag(),
        ..default()
      }
      .flaws(),
      vec![Flaw::Opcode, Flaw::Varint],
    );
  }
}
