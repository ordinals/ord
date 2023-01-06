use super::*;

// TODO:
// - always print vout bytes as two hex digits
// - print inscription ID on send
// - make parse recognize inscription IDs

pub(crate) type InscriptionIdArray = [u8; 32];

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct InscriptionId {
  pub(crate) txid: Txid,
  pub(crate) vout: u32,
}

impl InscriptionId {
  pub(crate) fn from_inner(inner: InscriptionIdArray) -> Self {
    Self {
      txid: Txid::from_inner(inner),
      vout: 0,
    }
  }

  pub(crate) fn as_inner(&self) -> &InscriptionIdArray {
    self.txid.as_inner()
  }

  pub(crate) fn into_inner(self) -> InscriptionIdArray {
    self.txid.into_inner()
  }
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
    write!(f, "{}{:x}", self.txid, self.vout)
  }
}

impl FromStr for InscriptionId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    todo!()
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

impl Encodable for InscriptionId {
  fn consensus_encode<W: io::Write + ?Sized>(&self, w: &mut W) -> Result<usize, io::Error> {
    let len = self.txid.consensus_encode(w)?;
    Ok(len + self.vout.consensus_encode(w)?)
  }
}

impl Decodable for InscriptionId {
  fn consensus_decode<R: io::Read + ?Sized>(
    r: &mut R,
  ) -> Result<Self, bitcoin::consensus::encode::Error> {
    Ok(Self {
      txid: Decodable::consensus_decode(r)?,
      vout: Decodable::consensus_decode(r)?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      inscription_id(1).to_string(),
      "11111111111111111111111111111111111111111111111111111111111111111",
    );
    assert_eq!(
      InscriptionId {
        txid: txid(1),
        vout: 0x0ABC,
      }
      .to_string(),
      "1111111111111111111111111111111111111111111111111111111111111111abc",
    );
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "11111111111111111111111111111111111111111111111111111111111111111"
        .parse::<InscriptionId>()
        .unwrap(),
      inscription_id(1),
    );
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111abc"
        .parse::<InscriptionId>()
        .unwrap(),
      InscriptionId {
        txid: txid(1),
        vout: 0x0ABC,
      },
    );
  }
}
