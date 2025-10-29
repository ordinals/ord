use {
  super::*,
  minicbor::{Decode, Decoder, Encode, Encoder, decode, encode},
};

#[derive(Decode, Default, Encode)]
#[cbor(map)]
pub(super) struct Attributes {
  #[n(0)]
  pub(super) title: Option<String>,
}

#[derive(Decode, Encode)]
#[cbor(map)]
pub(super) struct Item {
  #[n(0)]
  pub(super) id: Option<InscriptionId>,
  #[n(1)]
  pub(super) attributes: Option<Attributes>,
}

#[derive(Decode, Default, Encode)]
#[cbor(map)]
pub(super) struct Properties {
  #[n(0)]
  pub(super) gallery: Option<Vec<Item>>,
  #[n(1)]
  pub(super) attributes: Option<Attributes>,
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
