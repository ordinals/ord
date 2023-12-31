use serde::{Deserialize, Serialize};
use std::fmt::Display;

// the act of marking an inscription.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CollectionKind {
  BitMap,
}
impl Display for CollectionKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        CollectionKind::BitMap => String::from("bitmap"),
      }
    )
  }
}
