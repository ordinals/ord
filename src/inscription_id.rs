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
    write!(f, "{:08x}", self.vout)?;
    Ok(())
  }
}

impl FromStr for InscriptionId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    const TXID_LEN: usize = 64;
    const VOUT_LEN: usize = 8;
    const TOTAL_LEN: usize = TXID_LEN + VOUT_LEN;

    if s.len() != TOTAL_LEN {
      bail!("inscription ids must be {TOTAL_LEN}");
    }

    let (txid, vout) = s.split_at(TXID_LEN);

    // todo: Test
    Ok(Self {
      txid: txid.parse()?,
      vout: u32::from_str_radix(&vout, 16)?,
    })
  }
}

impl From<Txid> for InscriptionId {
  fn from(txid: Txid) -> Self {
    Self { txid, vout: 0 }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      inscription_id(1).to_string(),
      "111111111111111111111111111111111111111111111111111111111111111100000001",
    );
    assert_eq!(
      InscriptionId {
        txid: txid(1),
        vout: 0,
      }
      .to_string(),
      "111111111111111111111111111111111111111111111111111111111111111100000000",
    );
    assert_eq!(
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFFFFFF,
      }
      .to_string(),
      "1111111111111111111111111111111111111111111111111111111111111111ffffffff",
    );
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "111111111111111111111111111111111111111111111111111111111111111100000001"
        .parse::<InscriptionId>()
        .unwrap(),
      inscription_id(1),
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111ffffffff"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFFFFFF,
      },
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111FFFFFFFF"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFFFFFF,
      },
    );
  }
}
