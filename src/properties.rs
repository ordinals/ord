use {
  super::*,
  minicbor::{decode, encode, Decode, Decoder, Encode, Encoder},
};

#[derive(Debug, Default, PartialEq)]
pub struct Properties {
  pub(crate) gallery: Vec<InscriptionId>,
}

impl Properties {
  pub(crate) fn from_cbor(cbor: &[u8]) -> Self {
    let Ok(raw) = decode::<RawProperties>(cbor) else {
      return Self::default();
    };

    Self {
      gallery: raw
        .gallery
        .and_then(|gallery| {
          let mut items = Vec::new();

          for item in gallery {
            items.push(item.id?);
          }

          Some(items)
        })
        .unwrap_or_default(),
    }
  }

  pub(crate) fn to_cbor(&self) -> Option<Vec<u8>> {
    if *self == Self::default() {
      return None;
    }

    Some(
      minicbor::to_vec(RawProperties {
        gallery: Some(
          self
            .gallery
            .iter()
            .copied()
            .map(|item| GalleryItem { id: Some(item) })
            .collect(),
        ),
      })
      .unwrap(),
    )
  }
}

#[derive(Decode, Encode)]
#[cbor(map)]
pub(crate) struct GalleryItem {
  #[n(0)]
  pub(crate) id: Option<InscriptionId>,
}

#[derive(Decode, Encode)]
#[cbor(map)]
pub(crate) struct RawProperties {
  #[n(0)]
  pub(crate) gallery: Option<Vec<GalleryItem>>,
}

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
enum DecodeError {
  #[snafu(display("invalid inscription ID length {len}"))]
  InscriptionId { len: usize },
}

impl<'a, T> Decode<'a, T> for InscriptionId {
  fn decode(decoder: &mut Decoder<'a>, _: &mut T) -> Result<Self, decode::Error> {
    let bytes = decoder.bytes()?;

    Self::from_value(bytes)
      .ok_or_else(|| decode::Error::custom(InscriptionIdError { len: bytes.len() }.build()))
  }
}

impl<T> Encode<T> for InscriptionId {
  fn encode<W>(&self, encoder: &mut Encoder<W>, _: &mut T) -> Result<(), encode::Error<W::Error>>
  where
    W: encode::Write,
  {
    encoder.bytes(&self.value()).map(|_| ())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode() {
    assert_eq!(Properties::from_cbor(&[]), Properties::default());
  }

  #[test]
  fn encode() {
    assert_eq!(Properties::default().to_cbor(), None);

    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(2)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(0).value())
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(1).value())
        .unwrap();
    }

    let expected = Properties {
      gallery: vec![inscription_id(0), inscription_id(1)],
    };

    assert_eq!(expected.to_cbor(), Some(buffer.clone()));

    assert_eq!(Properties::from_cbor(&buffer), expected);
  }

  #[test]
  fn invalid_gallery_item_produces_empty_gallery() {
    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(2)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(0).value())
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&[1, 2, 3])
        .unwrap();
    }

    assert_eq!(Properties::from_cbor(&buffer), Properties::default());
  }
}
