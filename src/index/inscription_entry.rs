use super::*;

pub(crate) struct InscriptionEntry {
  pub(crate) height: u64,
  pub(crate) number: u64,
  pub(crate) sat: Option<Sat>,
  pub(crate) timestamp: u32,
}

pub(crate) type InscriptionEntryValue = (u64, u64, u64, u32);

impl InscriptionEntry {
  pub(crate) fn load((height, number, sat, timestamp): InscriptionEntryValue) -> Self {
    Self {
      height,
      number,
      sat: if sat == u64::MAX {
        None
      } else {
        Some(Sat(sat))
      },
      timestamp,
    }
  }

  pub(crate) fn store(self) -> InscriptionEntryValue {
    (
      self.height,
      self.number,
      match self.sat {
        Some(sat) => sat.n(),
        None => u64::MAX,
      },
      self.timestamp,
    )
  }
}
