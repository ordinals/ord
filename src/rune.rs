use {super::*, bitcoin::blockdata::script::Instructions};

// TODO:
// - make sure magic number is correct endianness
// - include which ordinals to inscribe the rune on
//  - single ordinal
//  - list of ordinals
//  - range of ordinals
//  - range of ordinals with step
// - post to endpoint on ordinals.com
// - display all runes ordinals.com
// - fetch a particular rune by hash at ordinals.com
// - display runes on ordinals page
//
// later:
// - issuing ordinal
// - image for rune
// - image for ordinals
// - content/media (include mime type)
// - flat format
//
// vint w
//
// ordinal privacy page
//
// - script
// - script bytes
// - result bytes

#[derive(Debug, Deserialize)]
pub(crate) struct Rune {
  pub(crate) magic: Network,
  pub(crate) name: String,
  pub(crate) ordinal: Ordinal,
}

impl MerkleScript for Rune {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    let mut object = BTreeMap::new();
    object.insert("m", Value::Bytes(self.magic.magic().to_le_bytes().to_vec()));
    object.insert("n", Value::String(self.name.clone()));
    object.insert("o", Value::Number(self.ordinal.n().into()));
    object.push_merkle_script(builder)
  }

  fn pop_merkle_script(instructions: &mut Instructions) -> Self
  where
    Self: Sized,
  {
    let tag = instructions.next().unwrap();
    todo!()
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
        ordinal: Ordinal(0),
      }
      .merkle_script()
      .asm(),
      concat! {
        "OP_PUSHNUM_5 OP_PUSHNUM_3",
        " OP_PUSHBYTES_1 6d OP_PUSHNUM_2 OP_PUSHBYTES_4 f9beb4d9",
        " OP_PUSHBYTES_1 6e OP_PUSHNUM_3 OP_PUSHBYTES_4 636f796e",
        " OP_PUSHBYTES_1 6f OP_PUSHNUM_1 OP_0",
      }
    );
  }
}
