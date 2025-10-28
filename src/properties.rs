use {
  super::*,
  minicbor::{Decode, Decoder, Encode, Encoder, decode, encode},
};

#[derive(Debug, Decode, Default, Encode, PartialEq)]
#[cbor(map)]
pub struct Properties {
  #[n(0)]
  gallery: Option<Vec<GalleryItem>>,
}

impl Properties {
  pub(crate) fn from_cbor(cbor: &[u8]) -> Self {
    decode(cbor).unwrap_or_default()
  }

  pub(crate) fn gallery(&self) -> Vec<InscriptionId> {
    let Some(gallery) = &self.gallery else {
      return Vec::new();
    };

    let mut ids = Vec::new();

    for item in gallery {
      let Some(id) = item.id else { return Vec::new() };

      ids.push(id);
    }

    ids
  }

  pub(crate) fn new(gallery: &[InscriptionId]) -> Self {
    Self {
      gallery: if gallery.is_empty() {
        None
      } else {
        Some(
          gallery
            .iter()
            .map(|id| GalleryItem { id: Some(*id) })
            .collect(),
        )
      },
    }
  }

  pub(crate) fn to_cbor(&self) -> Option<Vec<u8>> {
    if *self == Self::default() {
      return None;
    }

    Some(minicbor::to_vec(self).unwrap())
  }
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
      gallery: Some(vec![
        GalleryItem {
          id: Some(inscription_id(0)),
        },
        GalleryItem {
          id: Some(inscription_id(1)),
        },
      ]),
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
