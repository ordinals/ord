use super::*;

pub(crate) struct SatPoint {
  pub(crate) outpoint: OutPoint,
  pub(crate) offset: u64,
}

impl SatPoint {
  pub(crate) fn from_transaction_and_offset(tx: &Transaction, mut offset: u64) -> SatPoint {
    for (vout, output) in tx.output.iter().enumerate() {
      if output.value > offset {
        return SatPoint {
          outpoint: OutPoint {
            txid: tx.txid(),
            vout: vout.try_into().unwrap(),
          },
          offset,
        };
      }
      offset -= output.value;
    }

    panic!("Could not find ordinal in transaction!");
  }
}

impl Display for SatPoint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.outpoint, self.offset)
  }
}

impl Encodable for SatPoint {
  fn consensus_encode<S: io::Write>(&self, mut s: S) -> Result<usize, io::Error> {
    let len = self.outpoint.consensus_encode(&mut s)?;
    Ok(len + self.offset.consensus_encode(s)?)
  }
}

impl Decodable for SatPoint {
  fn consensus_decode<D: io::Read>(mut d: D) -> Result<Self, bitcoin::consensus::encode::Error> {
    Ok(SatPoint {
      outpoint: Decodable::consensus_decode(&mut d)?,
      offset: Decodable::consensus_decode(d)?,
    })
  }
}
