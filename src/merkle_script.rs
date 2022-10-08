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

impl MerkleScript for String {
  fn push_merkle_script(&self, builder: script::Builder) -> script::Builder {
    self.as_str().push_merkle_script(builder)
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
