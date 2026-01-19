use {
  super::*,
  minicbor::{
    Decode, Decoder, Encode, Encoder,
    data::{TryFromIntError, Type},
    decode, encode,
  },
  serde::{
    Serializer,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
  },
};

#[derive(Decode, Default, Encode)]
#[cbor(map)]
pub(super) struct Attributes {
  #[n(0)]
  pub(super) title: Option<String>,
  #[n(1)]
  pub(super) traits: Option<Traits>,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Trait {
  Bool(bool),
  Integer(i64),
  Null,
  String(String),
}

impl Display for Trait {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Bool(value) => write!(f, "{value}"),
      Self::Integer(value) => write!(f, "{value}"),
      Self::Null => write!(f, "null"),
      Self::String(value) => write!(f, "{value}"),
    }
  }
}

impl<'a, C> Decode<'a, C> for Trait {
  fn decode(decoder: &mut Decoder<'a>, _context: &mut C) -> Result<Self, decode::Error> {
    let ty = decoder.datatype()?;
    match ty {
      Type::Array
      | Type::Bytes
      | Type::F16
      | Type::F32
      | Type::F64
      | Type::Map
      | Type::Simple
      | Type::Tag
      | Type::Undefined => Err(decode::Error::custom(DecodeError::UnexpectedType { ty })),
      Type::ArrayIndef | Type::BytesIndef | Type::MapIndef | Type::StringIndef => {
        Err(decode::Error::custom(DecodeError::IndefiniteLengthType {
          ty,
        }))
      }
      Type::Bool => Ok(Self::Bool(decoder.bool()?)),
      Type::Break => Err(decode::Error::custom(DecodeError::UnexpectedBreak)),
      Type::I16
      | Type::I32
      | Type::I64
      | Type::I8
      | Type::Int
      | Type::U16
      | Type::U32
      | Type::U64
      | Type::U8 => Ok(Self::Integer(decoder.int()?.try_into().map_err(
        |source| decode::Error::custom(DecodeError::IntegerRange { source }),
      )?)),
      Type::Null => {
        decoder.null()?;
        Ok(Self::Null)
      }
      Type::String => Ok(Self::String(decoder.str()?.into())),
      Type::Unknown(byte) => Err(decode::Error::custom(DecodeError::UnknownType { ty: byte })),
    }
  }
}

impl<C> Encode<C> for Trait {
  fn encode<W>(&self, encoder: &mut Encoder<W>, _: &mut C) -> Result<(), encode::Error<W::Error>>
  where
    W: encode::Write,
  {
    match self {
      Self::Bool(value) => encoder.bool(*value),
      Self::Integer(value) => encoder.i64(*value),
      Self::Null => encoder.null(),
      Self::String(value) => encoder.str(value),
    }
    .map(|_| ())
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Traits {
  pub items: Vec<(String, Trait)>,
}

impl<'a, C> Decode<'a, C> for Traits {
  fn decode(decoder: &mut Decoder<'a>, _context: &mut C) -> Result<Self, decode::Error> {
    let len = decoder.map()?.ok_or_else(|| {
      decode::Error::custom(DecodeError::IndefiniteLengthType { ty: Type::MapIndef })
    })?;

    let mut items = Vec::new();

    let mut names = HashSet::new();
    for _ in 0..len {
      let name = decoder.decode::<String>()?;

      if !names.insert(name.clone()) {
        return Err(decode::Error::custom(DecodeError::DuplicateTrait { name }));
      }

      let value = decoder.decode::<Trait>()?;

      items.push((name, value));
    }

    Ok(Self { items })
  }
}

impl<C> Encode<C> for Traits {
  fn encode<W>(&self, encoder: &mut Encoder<W>, _: &mut C) -> Result<(), encode::Error<W::Error>>
  where
    W: encode::Write,
  {
    encoder.map(self.items.len().into_u64())?;

    for (name, value) in &self.items {
      encoder.str(name)?.encode(value)?;
    }

    Ok(())
  }
}

impl Serialize for Traits {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.items.len()))?;
    for (name, value) in &self.items {
      map.serialize_entry(name, value)?;
    }
    map.end()
  }
}

struct TraitsVisitor;

impl<'a> Visitor<'a> for TraitsVisitor {
  type Value = Traits;

  fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("map of strings to traits")
  }

  fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'a>,
  {
    let mut items = Vec::new();

    let mut names = HashSet::new();
    while let Some((name, value)) = access.next_entry::<String, Trait>()? {
      if !names.insert(name.clone()) {
        use serde::de::Error;
        return Err(M::Error::custom(format!("duplicate trait {name}")));
      }
      items.push((name, value));
    }

    Ok(Traits { items })
  }
}

impl<'a> Deserialize<'a> for Traits {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'a>,
  {
    deserializer.deserialize_map(TraitsVisitor)
  }
}

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
enum DecodeError {
  #[snafu(display("duplicate trait `{name}`"))]
  DuplicateTrait { name: String },
  #[snafu(display("indefinite length types are not allowed: {ty}"))]
  IndefiniteLengthType { ty: Type },
  #[snafu(display("invalid inscription ID length {len}"))]
  InscriptionId { len: usize },
  #[snafu(display("integer out of range"))]
  IntegerRange { source: TryFromIntError },
  #[snafu(display("unexpected break"))]
  UnexpectedBreak,
  #[snafu(display("unexpected type: {ty}"))]
  UnexpectedType { ty: Type },
  #[snafu(display("unknown type: {ty:x}"))]
  UnknownType { ty: u8 },
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
