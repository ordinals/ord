use super::*;

pub type ContentHashValue = [u8; 32];

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub struct ContentHash {
  pub hash: ContentHashValue,
}

impl FromStr for ContentHash {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let bytes = hex::decode(s).context("Failed to decode hex encoded hash")?;

    if bytes.len() != 32 {
      anyhow::bail!("Invalid hash length {}, require 32 bytes", bytes.len());
    }

    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);

    Ok(Self { hash: array })
  }
}

impl<'de> Deserialize<'de> for ContentHash {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}

impl Serialize for ContentHash {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl Display for ContentHash {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", hex::encode(self.hash))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      ContentHash { hash: [0u8; 32] }.to_string(),
      "0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert_eq!(
      ContentHash {
        hash: [
          0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
          0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
          0x1F, 0x20,
        ]
      }
      .to_string(),
      "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
    );
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "0000000000000000000000000000000000000000000000000000000000000000"
        .parse::<ContentHash>()
        .unwrap(),
      ContentHash { hash: [0u8; 32] },
    );
    assert_eq!(
      "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
        .parse::<ContentHash>()
        .unwrap(),
      ContentHash {
        hash: [
          0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
          0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
          0x1F, 0x20,
        ]
      },
    );
  }

  #[test]
  fn from_str_bad_character() {
    assert_matches!(
      "0000000000000000000000000000000000000000000000000000000000000g".parse::<ContentHash>(),
      Err(anyhow::Error { .. }),
    );
  }

  #[test]
  fn from_str_bad_length() {
    assert_matches!(
      "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021".parse::<ContentHash>(),
      Err(anyhow::Error { .. })
    );
  }
}
