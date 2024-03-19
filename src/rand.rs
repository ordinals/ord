use super::*;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct RuneRand {
  pub rune_id: RuneId,
  pub txid: Txid,
  pub tx_index: usize,
  pub timestamp: u32,
}

impl RuneRand {
  pub(super) fn hash(self) -> u64 {
    let combined_tx = format!("{}{}", self.txid, self.tx_index);
    let mut hasher = DefaultHasher::new();
    combined_tx.hash(&mut hasher);

    self.timestamp.hash(&mut hasher);
    self.rune_id.hash(&mut hasher);

    hasher.finish()
  }

  pub(super) fn get_output(self, output_len: usize) -> usize {
    let hash = self.hash();

    usize::try_from(hash).unwrap() % output_len
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash() {
    let tx_hash: &str = "02181b0ed03e4a7ee9909f470011e4aa43fe675b4671bdefa029b17b0749ae79";
    let rune_rand = RuneRand {
      rune_id: RuneId::default(),
      txid: Txid::from_str(tx_hash).unwrap(),
      tx_index: 0,
      timestamp: 123,
    };

    assert_eq!(rune_rand.hash(), 16229684257583859262);
  }
}
