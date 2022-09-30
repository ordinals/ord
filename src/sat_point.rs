use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct SatPoint {
  pub(crate) outpoint: OutPoint,
  pub(crate) offset: u64,
}

impl Display for SatPoint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.outpoint, self.offset)
  }
}

impl Encodable for SatPoint {
  fn consensus_encode<S: io::Write + ?Sized>(&self, s: &mut S) -> Result<usize, io::Error> {
    let len = self.outpoint.consensus_encode(s)?;
    Ok(len + self.offset.consensus_encode(s)?)
  }
}

impl Decodable for SatPoint {
  fn consensus_decode<D: io::Read + ?Sized>(
    d: &mut D,
  ) -> Result<Self, bitcoin::consensus::encode::Error> {
    Ok(SatPoint {
      outpoint: Decodable::consensus_decode(d)?,
      offset: Decodable::consensus_decode(d)?,
    })
  }
}
