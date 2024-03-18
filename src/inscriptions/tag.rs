use super::*;

#[derive(Copy, Clone)]
#[repr(u8)]
pub(crate) enum Tag {
  Pointer = 2,
  #[allow(unused)]
  Unbound = 66,

  ContentType = 1,
  Parent = 3,
  Metadata = 5,
  Metaprotocol = 7,
  ContentEncoding = 9,
  Delegate = 11,
  Rune = 13,
  #[allow(unused)]
  Note = 15,
  #[allow(unused)]
  Nop = 255,
}

impl Tag {
  fn chunked(self) -> bool {
    matches!(self, Self::Metadata)
  }

  pub(crate) fn bytes(self) -> [u8; 1] {
    [self as u8]
  }

  pub(crate) fn append(self, builder: &mut script::Builder, value: &Option<Vec<u8>>) {
    if let Some(value) = value {
      let mut tmp = script::Builder::new();
      mem::swap(&mut tmp, builder);

      if self.chunked() {
        for chunk in value.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
          tmp = tmp
            .push_slice::<&script::PushBytes>(self.bytes().as_slice().try_into().unwrap())
            .push_slice::<&script::PushBytes>(chunk.try_into().unwrap());
        }
      } else {
        tmp = tmp
          .push_slice::<&script::PushBytes>(self.bytes().as_slice().try_into().unwrap())
          .push_slice::<&script::PushBytes>(value.as_slice().try_into().unwrap());
      }

      mem::swap(&mut tmp, builder);
    }
  }

  pub(crate) fn append_array(self, builder: &mut script::Builder, values: &Vec<Vec<u8>>) {
    let mut tmp = script::Builder::new();
    mem::swap(&mut tmp, builder);

    for value in values {
      tmp = tmp
        .push_slice::<&script::PushBytes>(self.bytes().as_slice().try_into().unwrap())
        .push_slice::<&script::PushBytes>(value.as_slice().try_into().unwrap());
    }

    mem::swap(&mut tmp, builder);
  }

  pub(crate) fn take(self, fields: &mut BTreeMap<&[u8], Vec<&[u8]>>) -> Option<Vec<u8>> {
    if self.chunked() {
      let value = fields.remove(self.bytes().as_slice())?;

      if value.is_empty() {
        None
      } else {
        Some(value.into_iter().flatten().cloned().collect())
      }
    } else {
      let values = fields.get_mut(self.bytes().as_slice())?;

      if values.is_empty() {
        None
      } else {
        let value = values.remove(0).to_vec();

        if values.is_empty() {
          fields.remove(self.bytes().as_slice());
        }

        Some(value)
      }
    }
  }

  pub(crate) fn take_array(self, fields: &mut BTreeMap<&[u8], Vec<&[u8]>>) -> Vec<Vec<u8>> {
    fields
      .remove(self.bytes().as_slice())
      .unwrap_or_default()
      .into_iter()
      .map(|v| v.to_vec())
      .collect()
  }
}
