use super::*;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResumeEntry {
  pub commit: Transaction,
  pub reveal: Transaction,
}

pub(super) type ResumeEntryValue = (Vec<u8>, Vec<u8>);

impl Entry for ResumeEntry {
  type Value = ResumeEntryValue;

  fn load((commit, reveal): ResumeEntryValue) -> Self {
    Self {
      commit: consensus::encode::deserialize::<Transaction>(&commit).unwrap(),
      reveal: consensus::encode::deserialize::<Transaction>(&reveal).unwrap(),
    }
  }

  fn store(self) -> Self::Value {
    (
      consensus::encode::serialize(&self.commit),
      consensus::encode::serialize(&self.reveal),
    )
  }
}
