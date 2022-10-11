use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Rune {
  pub(crate) name: String,
  pub(crate) chain: Chain,
  pub(crate) ordinal: Ordinal,
}
