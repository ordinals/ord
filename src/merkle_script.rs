use super::*;

#[repr(i64)]
enum Tag {
  // Hash = 0,
  // Number = 1,
  Bytes = 2,
  String = 3,
  // Array = 4,
  Object = 5,
}

pub(crate) enum Value {
  String(String),
  Bytes(Vec<u8>),
}

pub(crate) trait MerkleScript {
  fn merkle_script(&self) -> Script {
    self
      .push_merkle_script(script::Builder::new())
      .into_script()
  }

  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder;
}

impl MerkleScript for str {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    builder
      .push_int(Tag::String as i64)
      .push_slice(self.as_ref())
  }
}

impl MerkleScript for &str {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    (*self).push_merkle_script(builder)
  }
}

impl MerkleScript for [u8] {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    builder.push_int(Tag::Bytes as i64).push_slice(self)
  }
}

impl MerkleScript for Value {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    match self {
      Self::String(string) => string.push_merkle_script(builder),
      Self::Bytes(bytes) => bytes.push_merkle_script(builder),
    }
  }
}

impl<T: MerkleScript> MerkleScript for BTreeMap<&str, T> {
  fn push_merkle_script(&self, mut builder: script::Builder) -> script::Builder {
    builder = builder
      .push_int(Tag::Object as i64)
      .push_int(self.len().try_into().unwrap());

    for (key, value) in self {
      builder = value.push_merkle_script(builder.push_slice(key.as_ref()));
    }

    builder
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn string() {
    assert_eq!(
      "foo".merkle_script().asm(),
      "OP_PUSHNUM_3 OP_PUSHBYTES_3 666f6f",
    );
    assert_eq!("".merkle_script().asm(), "OP_PUSHNUM_3 OP_0",);
  }

  #[test]
  fn value_string() {
    assert_eq!(
      Value::String("foo".into()).merkle_script().asm(),
      "OP_PUSHNUM_3 OP_PUSHBYTES_3 666f6f",
    );
  }

  #[test]
  fn bytes() {
    assert_eq!(
      [1, 2, 3].merkle_script().asm(),
      "OP_PUSHNUM_2 OP_PUSHBYTES_3 010203",
    );
    assert_eq!([].merkle_script().asm(), "OP_PUSHNUM_2 OP_0",);
  }

  #[test]
  fn value_bytes() {
    assert_eq!(
      Value::Bytes(vec![1, 2, 3]).merkle_script().asm(),
      "OP_PUSHNUM_2 OP_PUSHBYTES_3 010203",
    );
  }

  #[test]
  fn object() {
    assert_eq!(
      [("a", "A"), ("b", "B"), ("c", "C")]
        .into_iter()
        .collect::<BTreeMap<&str, &str>>()
        .merkle_script()
        .asm(),
      concat!(
        "OP_PUSHNUM_5 OP_PUSHNUM_3",
        " OP_PUSHBYTES_1 61 OP_PUSHNUM_3 OP_PUSHBYTES_1 41",
        " OP_PUSHBYTES_1 62 OP_PUSHNUM_3 OP_PUSHBYTES_1 42",
        " OP_PUSHBYTES_1 63 OP_PUSHNUM_3 OP_PUSHBYTES_1 43",
      ),
    );
  }
}
