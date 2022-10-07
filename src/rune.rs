use super::*;

use bitcoin::{blockdata::script, Script};

#[repr(i64)]
enum Tag {
  Hash = 0,
  Integer = 1,
  Bytes = 2,
  Text = 3,
  Array = 4,
  Object = 5,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Rune {}

impl Rune {
  fn merkle_script(&self) -> Script {
    script::Builder::new()
      .push_int(Tag::Object as i64)
      .push_int(0)
      .into_script()
  }
}

impl FromStr for Rune {
  type Err = serde_json::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    serde_json::from_str(s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_encoding() {
    assert_eq!(Rune {}.merkle_script().asm(), "OP_PUSHNUM_5 OP_0");
  }
}
