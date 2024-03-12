use super::*;

#[derive(Copy, Clone, Debug)]
pub(super) enum Tag {
  Body = 0,
  Flags = 2,
  Rune = 4,
  Limit = 6,
  Term = 8,
  Deadline = 10,
  DefaultOutput = 12,
  Claim = 14,
  #[allow(unused)]
  Burn = 126,

  Divisibility = 1,
  Spacers = 3,
  Symbol = 5,
  #[allow(unused)]
  Nop = 127,
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_u128() {
    assert_eq!(0u128, Tag::Body.into());
    assert_eq!(2u128, Tag::Flags.into());
  }

  #[test]
  fn partial_eq() {
    assert_eq!(Tag::Body, 0);
    assert_eq!(Tag::Flags, 2);
  }

  #[test]
  fn take() {
    let mut fields = vec![(2, 3)].into_iter().collect::<HashMap<u128, u128>>();

    assert_eq!(Tag::Flags.take(&mut fields), Some(3));

    assert!(fields.is_empty());

    assert_eq!(Tag::Flags.take(&mut fields), None);
  }

  #[test]
  fn encode() {
    let mut payload = Vec::new();

    Tag::Flags.encode(3, &mut payload);

    assert_eq!(payload, [2, 3]);

    Tag::Rune.encode(5, &mut payload);

    assert_eq!(payload, [2, 3, 4, 5]);
  }

  #[test]
  fn burn_and_nop_are_one_byte() {
    let mut payload = Vec::new();
    Tag::Burn.encode(0, &mut payload);
    assert_eq!(payload.len(), 2);

    let mut payload = Vec::new();
    Tag::Nop.encode(0, &mut payload);
    assert_eq!(payload.len(), 2);
  }
}
