use super::*;

#[derive(Copy, Clone)]
pub(super) enum Tag {
  Body = 0,
  Flags = 2,
  Rune = 4,
  Limit = 6,
  Term = 8,
  Deadline = 10,
  DefaultOutput = 12,
  #[allow(unused)]
  Burn = 254,

  Divisibility = 1,
  Spacers = 3,
  Symbol = 5,
  #[allow(unused)]
  Nop = 255,
}

impl Tag {
  pub(super) fn take(self, fields: &mut HashMap<u128, u128>) -> Option<u128> {
    fields.remove(&self.into())
  }

  pub(super) fn encode(self, value: u128, payload: &mut Vec<u8>) {
    varint::encode_to_vec(self.into(), payload);
    varint::encode_to_vec(value, payload);
  }
}

impl From<Tag> for u128 {
  fn from(tag: Tag) -> Self {
    tag as u128
  }
}

impl PartialEq<u128> for Tag {
  fn eq(&self, other: &u128) -> bool {
    u128::from(*self) == *other
  }
}
