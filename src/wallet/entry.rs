use super::*;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EtchingEntry {
  pub commit: Transaction,
  pub reveal: Transaction,
  pub output: batch::Output,
}

pub(super) type EtchingEntryValue = (
  Vec<u8>, // commit
  Vec<u8>, // reveal
  Vec<u8>, // output
);

impl Entry for EtchingEntry {
  type Value = EtchingEntryValue;

  fn load((commit, reveal, output): EtchingEntryValue) -> Self {
    Self {
      commit: consensus::encode::deserialize::<Transaction>(&commit).unwrap(),
      reveal: consensus::encode::deserialize::<Transaction>(&reveal).unwrap(),
      output: serde_json::from_slice(&output).unwrap(),
    }
  }

  fn store(self) -> Self::Value {
    (
      consensus::encode::serialize(&self.commit),
      consensus::encode::serialize(&self.reveal),
      serde_json::to_string(&self.output)
        .unwrap()
        .as_bytes()
        .to_owned(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn etching_entry() {
    let commit = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      }],
      output: Vec::new(),
    };

    let reveal = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::default(),
        witness: Witness::new(),
      }],
      output: Vec::new(),
    };

    let txid = Txid::from_byte_array([
      0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
      0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D,
      0x1E, 0x1F,
    ]);

    let output = batch::Output {
      commit: txid,
      commit_psbt: None,
      inscriptions: Vec::new(),
      parents: Vec::new(),
      reveal: txid,
      reveal_broadcast: true,
      reveal_psbt: None,
      rune: None,
      total_fees: 0,
    };

    let value = (
      consensus::encode::serialize(&commit),
      consensus::encode::serialize(&reveal),
      serde_json::to_string(&output)
        .unwrap()
        .as_bytes()
        .to_owned(),
    );

    let entry = EtchingEntry {
      commit,
      reveal,
      output,
    };

    assert_eq!(entry.clone().store(), value);
    assert_eq!(EtchingEntry::load(value), entry);
  }
}
