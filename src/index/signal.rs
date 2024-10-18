use super::*;

/// A signal is emitted by a UTXO. It should have strict size constraint
/// Should there be a content type?
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Signal {
  pub message: Vec<u8>,
  pub signature: Witness,
}

// This is wrong it should be (Vec<u8>, Vec<Vec<u8>>)
// Running into redb issues
pub(crate) type SignalEntry = (Vec<u8>, Vec<u8>);

impl Entry for Signal {
  type Value = SignalEntry;

  fn load(value: Self::Value) -> Self {
    Self {
      message: value.0.to_vec(),
      signature: Witness::from_slice(&[&value.1]),
    }
  }

  fn store(self) -> Self::Value {
    (
      self.message,
      self.signature.to_vec().first().unwrap().clone(),
    )
  }
}
