use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Rune {
  pub(crate) name: String,
  pub(crate) network: Network,
  pub(crate) ordinal: Ordinal,
}
