use super::*;

// TODO:
// - fix tests
// - use TXIDiINSCRIPTION_NUMBER
// - print inscription ID on send
// - make parse recognize inscription IDs
// - make sure we use inscriptionIDs in index methods

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct InscriptionId {
  pub(crate) txid: Txid,
  // TODO: rename to index, i, or inscription
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
    write!(f, "{}i{}", self.txid, self.vout)
  }
}

impl FromStr for InscriptionId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    if !s.is_ascii() {
      bail!("invalid character");
    }

    const TXID_LEN: usize = 64;
    const MIN_LEN: usize = TXID_LEN + 2;

    if s.len() < MIN_LEN {
      bail!("invalid length");
    }

    let txid = &s[..TXID_LEN];

    if &s[TXID_LEN..TXID_LEN + 1] != "i" {
      bail!("invalid separator");
    }

    let vout = &s[TXID_LEN + 1..];

    Ok(Self {
      txid: txid.parse()?,
      vout: vout.parse()?,
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
      "1111111111111111111111111111111111111111111111111111111111111111i1",
    );
    assert_eq!(
      InscriptionId {
        txid: txid(1),
        vout: 0,
      }
      .to_string(),
      "1111111111111111111111111111111111111111111111111111111111111111i0",
    );
    assert_eq!(
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFFFFFF,
      }
      .to_string(),
      "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
    );
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111i1"
        .parse::<InscriptionId>()
        .unwrap(),
      inscription_id(1),
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111i4294967295"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFFFFFF,
      },
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111i4294967295"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0xFFFFFFFF,
      },
    );
  }
}
