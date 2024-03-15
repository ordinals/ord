use super::*;

#[derive(Copy, Clone)]
pub(crate) enum Tag {
  Pointer,
  #[allow(unused)]
  Unbound,

  ContentType,
  Parent,
  Metadata,
  Metaprotocol,
  ContentEncoding,
  Delegate,
  #[allow(unused)]
  Note,
  #[allow(unused)]
  Nop,
}

enum TagCodecStrategy {
  First,
  Chunked,
  Array,
}

impl Tag {
  fn parsing_strategy(self) -> TagCodecStrategy {
    match self {
      Tag::Metadata => TagCodecStrategy::Chunked,
      Tag::Parent => TagCodecStrategy::Array,
      _ => TagCodecStrategy::First,
    }
  }

  pub(crate) fn bytes(self) -> &'static [u8] {
    match self {
      Self::Pointer => &[2],
      Self::Unbound => &[66],

      Self::ContentType => &[1],
      Self::Parent => &[3],
      Self::Metadata => &[5],
      Self::Metaprotocol => &[7],
      Self::ContentEncoding => &[9],
      Self::Delegate => &[11],
      Self::Note => &[15],
      Self::Nop => &[255],
    }
  }

  pub(crate) fn append(self, builder: &mut script::Builder, value: &Option<Vec<u8>>) {
    if let Some(value) = value {
      let mut tmp = script::Builder::new();
      mem::swap(&mut tmp, builder);

      match self.parsing_strategy() {
        TagCodecStrategy::First | TagCodecStrategy::Array => {
          tmp = tmp
            .push_slice::<&script::PushBytes>(self.bytes().try_into().unwrap())
            .push_slice::<&script::PushBytes>(value.as_slice().try_into().unwrap());
        }
        TagCodecStrategy::Chunked => {
          for chunk in value.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
            tmp = tmp
              .push_slice::<&script::PushBytes>(self.bytes().try_into().unwrap())
              .push_slice::<&script::PushBytes>(chunk.try_into().unwrap());
          }
        }
      }

      mem::swap(&mut tmp, builder);
    }
  }

  pub(crate) fn append_array(self, builder: &mut script::Builder, values: &Vec<Vec<u8>>) {
    let mut tmp = script::Builder::new();
    mem::swap(&mut tmp, builder);

    for value in values {
      tmp = tmp
        .push_slice::<&script::PushBytes>(self.bytes().try_into().unwrap())
        .push_slice::<&script::PushBytes>(value.as_slice().try_into().unwrap());
    }

    mem::swap(&mut tmp, builder);
  }

  pub(crate) fn take(self, fields: &mut BTreeMap<&[u8], Vec<&[u8]>>) -> Option<Vec<u8>> {
    match self.parsing_strategy() {
      TagCodecStrategy::First => {
        let values = fields.get_mut(self.bytes())?;

        if values.is_empty() {
          None
        } else {
          let value = values.remove(0).to_vec();

          if values.is_empty() {
            fields.remove(self.bytes());
          }

          Some(value)
        }
      }
      TagCodecStrategy::Chunked => {
        let value = fields.remove(self.bytes())?;

        if value.is_empty() {
          None
        } else {
          Some(value.into_iter().flatten().cloned().collect())
        }
      }
      TagCodecStrategy::Array => {
        panic!("Array-type fields must not be removed as a simple byte array.")
      }
    }
  }

  pub(crate) fn take_array(self, fields: &mut BTreeMap<&[u8], Vec<&[u8]>>) -> Vec<Vec<u8>> {
    let values = fields.remove(self.bytes()).unwrap_or_default();
    values.into_iter().map(|v| v.to_vec()).collect()
  }
}
