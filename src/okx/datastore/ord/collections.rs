use serde::{Deserialize, Serialize};

// the act of marking an inscription.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CollectionKind {
  BitMap,
}
impl ToString for CollectionKind {
  fn to_string(&self) -> String {
    match self {
      CollectionKind::BitMap => String::from("bitmap"),
    }
  }
}
