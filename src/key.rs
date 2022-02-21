use super::*;

pub(crate) struct Key {
  pub(crate) ordinal: u64,
  pub(crate) block: u64,
  pub(crate) transaction: u64,
}

impl Key {
  pub(crate) fn new(ordinal: Ordinal) -> Key {
    Self {
      ordinal: ordinal.0,
      block: u64::max_value(),
      transaction: u64::max_value(),
    }
  }

  pub(crate) fn encode(self) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(self.ordinal.to_be_bytes());
    buffer.extend(self.block.to_be_bytes());
    buffer.extend(self.transaction.to_be_bytes());
    buffer
  }

  pub(crate) fn decode(buffer: &[u8]) -> Result<Self> {
    if buffer.len() != 24 {
      return Err("Buffer too small to decode key from".into());
    }

    Ok(Key {
      ordinal: u64::from_be_bytes(buffer[0..8].try_into().unwrap()),
      block: u64::from_be_bytes(buffer[8..16].try_into().unwrap()),
      transaction: u64::from_be_bytes(buffer[16..24].try_into().unwrap()),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_error() {
    assert_eq!(
      Key::decode(&[]).err().unwrap().to_string(),
      "Buffer too small to decode key from"
    );
  }
}
