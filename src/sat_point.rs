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
