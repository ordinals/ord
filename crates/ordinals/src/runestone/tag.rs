use super::*;

#[derive(Copy, Clone, Debug)]
pub(super) enum Tag {
  Body = 0,
  Flags = 2,
  Rune = 4,
  Premine = 6,
  Cap = 8,
  Amount = 10,
  HeightStart = 12,
  HeightEnd = 14,
  OffsetStart = 16,
  OffsetEnd = 18,
  Mint = 20,
  Pointer = 22,
  #[allow(unused)]
  Cenotaph = 126,

  Divisibility = 1,
  Spacers = 3,
  Symbol = 5,
  #[allow(unused)]
  Nop = 127,
}

impl Tag {
  pub(super) fn take<const N: usize, T>(
    self,
    fields: &mut HashMap<u128, VecDeque<u128>>,
    with: impl Fn([u128; N]) -> Option<T>,
  ) -> Option<T> {
    let field = fields.get_mut(&self.into())?;

    let mut values: [u128; N] = [0; N];

    for (i, v) in values.iter_mut().enumerate() {
      *v = *field.get(i)?;
    }

    let value = with(values)?;

    field.drain(0..N);

    if field.is_empty() {
      fields.remove(&self.into()).unwrap();
    }

    Some(value)
  }

  pub(super) fn encode<const N: usize>(self, values: [u128; N], payload: &mut Vec<u8>) {
    for value in values {
      varint::encode_to_vec(self.into(), payload);
      varint::encode_to_vec(value, payload);
    }
  }

  pub(super) fn encode_option<T: Into<u128>>(self, value: Option<T>, payload: &mut Vec<u8>) {
    if let Some(value) = value {
      self.encode([value.into()], payload)
    }
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
    let mut fields = vec![(2, vec![3].into_iter().collect())]
      .into_iter()
      .collect::<HashMap<u128, VecDeque<u128>>>();

    assert_eq!(Tag::Flags.take(&mut fields, |[_]| None::<u128>), None);

    assert!(!fields.is_empty());

    assert_eq!(Tag::Flags.take(&mut fields, |[flags]| Some(flags)), Some(3));

    assert!(fields.is_empty());

    assert_eq!(Tag::Flags.take(&mut fields, |[flags]| Some(flags)), None);
  }

  #[test]
  fn take_leaves_unconsumed_values() {
    let mut fields = vec![(2, vec![1, 2, 3].into_iter().collect())]
      .into_iter()
      .collect::<HashMap<u128, VecDeque<u128>>>();

    assert_eq!(fields[&2].len(), 3);

    assert_eq!(Tag::Flags.take(&mut fields, |[_]| None::<u128>), None);

    assert_eq!(fields[&2].len(), 3);

    assert_eq!(
      Tag::Flags.take(&mut fields, |[a, b]| Some((a, b))),
      Some((1, 2))
    );

    assert_eq!(fields[&2].len(), 1);

    assert_eq!(Tag::Flags.take(&mut fields, |[a]| Some(a)), Some(3));

    assert_eq!(fields.get(&2), None);
  }

  #[test]
  fn encode() {
    let mut payload = Vec::new();

    Tag::Flags.encode([3], &mut payload);

    assert_eq!(payload, [2, 3]);

    Tag::Rune.encode([5], &mut payload);

    assert_eq!(payload, [2, 3, 4, 5]);

    Tag::Rune.encode([5, 6], &mut payload);

    assert_eq!(payload, [2, 3, 4, 5, 4, 5, 4, 6]);
  }

  #[test]
  fn burn_and_nop_are_one_byte() {
    let mut payload = Vec::new();
    Tag::Cenotaph.encode([0], &mut payload);
    assert_eq!(payload.len(), 2);

    let mut payload = Vec::new();
    Tag::Nop.encode([0], &mut payload);
    assert_eq!(payload.len(), 2);
  }
}
