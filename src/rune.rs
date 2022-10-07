use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct Rune {}

impl FromStr for Rune {
  type Err = serde_json::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    serde_json::from_str(s)
  }
}
