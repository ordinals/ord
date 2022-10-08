use super::*;

// TODO:
// - make sure magic number is current endianness

#[derive(Debug, Deserialize)]
pub(crate) struct Rune {
  pub(crate) magic: Network,
  pub(crate) name: String,
}

impl MerkleScript for Rune {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    let mut object = BTreeMap::new();
    object.insert("m", Value::Bytes(self.magic.magic().to_le_bytes().to_vec()));
    object.insert("n", Value::String(self.name.clone()));
    object.push_merkle_script(builder)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_eq!(
      Rune {
        magic: Network::Bitcoin,
        name: "coyn".into(),
      }
      .merkle_script()
      .asm(),
      concat! {
        "OP_PUSHNUM_5 OP_PUSHNUM_2",
        " OP_PUSHBYTES_1 6d OP_PUSHNUM_2 OP_PUSHBYTES_4 f9beb4d9",
        " OP_PUSHBYTES_1 6e OP_PUSHNUM_3 OP_PUSHBYTES_4 636f796e",
      }
    );
  }
}
