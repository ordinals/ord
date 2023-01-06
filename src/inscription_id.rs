use super::*;

// TODO:
// - always print vout bytes as two hex digits
// - print inscription ID on send
// - make parse recognize inscription IDs

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct InscriptionId {
  pub(crate) txid: Txid,
  pub(crate) vout: u32,
}

impl<'de> Deserialize<'de> for InscriptionId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}

impl Display for InscriptionId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.txid)?;

    for byte in self
      .vout
      .to_be_bytes()
      .into_iter()
      .skip_while(|byte| *byte == 0)
    {
      write!(f, "{:02x}", byte)?;
    }

    Ok(())
  }
}

impl FromStr for InscriptionId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    // todo: Test
    Ok(Self {
      txid: s[0..64].parse()?,
      vout: u32::from_str_radix(&s[64..], 16)?,
    })
  }
}

// todo: remove this
impl From<Txid> for InscriptionId {
  fn from(txid: Txid) -> Self {
    Self { txid, vout: 0 }
  }
}

// todo: remove this
impl PartialEq<Txid> for InscriptionId {
  fn eq(&self, txid: &Txid) -> bool {
    self.txid == *txid
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      inscription_id(1).to_string(),
      "111111111111111111111111111111111111111111111111111111111111111101",
    );
    assert_eq!(
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFF,
      }
      .to_string(),
      "1111111111111111111111111111111111111111111111111111111111111111ffff",
    );
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "111111111111111111111111111111111111111111111111111111111111111101"
        .parse::<InscriptionId>()
        .unwrap(),
      inscription_id(1),
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111ffff"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFF,
      },
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111FFFF"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFF,
      },
    );
  }
}
