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

#[derive(Clone, Debug, Decode, Default, Deserialize, Encode, PartialEq, Serialize)]
#[cbor(map)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
  #[cbor(n(0))]
  pub title: Option<String>,
  #[cbor(n(1), default, skip_if = "is_default")]
  #[serde(default)]
  pub traits: Traits,
}

#[derive(Clone, Debug, Decode, Default, Deserialize, Encode, PartialEq, Serialize)]
#[cbor(map)]
#[serde(deny_unknown_fields)]
pub struct Item {
  #[cbor(n(0))]
  pub id: Option<InscriptionId>,
  #[cbor(n(1), default, skip_if = "is_default")]
  pub attributes: Attributes,
  #[cbor(n(2))]
  #[serde(default, skip)]
  pub index: Option<u32>,
}

impl Item {
  pub(crate) fn id(&self) -> InscriptionId {
    self.id.unwrap()
  }
}

#[derive(Clone, Debug, Decode, Default, Deserialize, Encode, PartialEq, Serialize)]
#[cbor(map)]
#[serde(deny_unknown_fields)]
pub struct Properties {
  #[cbor(n(0), default, skip_if = "Vec::is_empty")]
  pub gallery: Vec<Item>,
  #[cbor(n(1), default, skip_if = "is_default")]
  pub attributes: Attributes,
  #[cbor(n(2), default, skip_if = "Vec::is_empty", with = "minicbor::bytes")]
  #[serde(default, skip)]
  pub txids: Vec<u8>,
}

impl Properties {
  pub(crate) fn from_cbor(cbor: &[u8]) -> Self {
    let mut properties = minicbor::decode::<Self>(cbor).unwrap_or_default();

    for (item, txid) in properties
      .gallery
      .iter_mut()
      .zip(properties.txids.as_chunks::<32>().0)
    {
      item.id = Some(InscriptionId {
        txid: Txid::from_slice(txid).unwrap(),
        index: item.index.unwrap_or_default(),
      });
    }

    properties.txids = Vec::new();

    for item in &mut properties.gallery {
      item.index = None;
    }

    if properties.gallery.iter().any(|item| item.id.is_none()) {
      properties.gallery = Vec::new();
    }

    properties
  }

  pub(crate) fn to_inline_cbor(&self) -> Option<Vec<u8>> {
    if *self == Self::default() {
      return None;
    }

    Some(minicbor::to_vec(self).unwrap())
  }

  pub(crate) fn to_packed_cbor(&self) -> Option<Vec<u8>> {
    let Properties {
      attributes,
      gallery,
      txids,
    } = self;

    assert!(txids.is_empty());

    if *self == Self::default() {
      return None;
    }

    let mut txids = Vec::with_capacity(gallery.len() * 32);
    let mut packed_gallery = Vec::with_capacity(gallery.len());

    for item in gallery {
      let Item {
        id,
        attributes,
        index,
      } = item;

      assert!(index.is_none());

      let id = id.unwrap();
      txids.extend_from_slice(&id.txid.to_byte_array());

      packed_gallery.push(Item {
        id: None,
        attributes: attributes.clone(),
        index: if id.index == 0 { None } else { Some(id.index) },
      });
    }

    let packed = Properties {
      gallery: packed_gallery,
      attributes: attributes.clone(),
      txids,
    };

    Some(minicbor::to_vec(&packed).unwrap())
  }
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

#[cfg(test)]
mod tests {
  use {super::*, minicbor::Encoder};

  #[test]
  fn decode() {
    assert_eq!(Properties::from_cbor(&[]), Properties::default());
  }

  #[test]
  fn encode() {
    assert_eq!(Properties::default().to_inline_cbor(), None);
    assert_eq!(Properties::default().to_packed_cbor(), None);

    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(3)
        .unwrap()

        // item 0
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(0).value())
        .unwrap()
        .u8(1)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .str("bar")
        .unwrap()

        // item 1
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(1).value())
        .unwrap()
        .u8(1)
        .unwrap()
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .str("baz")
        .unwrap()
        .u8(1)
        .unwrap()
        .map(1)
        .unwrap()
        .str("abc")
        .unwrap()
        .str("xyz")
        .unwrap()

        // item 2
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(2).value())
        .unwrap()

        // attributes
        .u8(1)
        .unwrap()
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .str("foo")
        .unwrap()
        .u8(1)
        .unwrap()
        .map(1)
        .unwrap()
        .str("hello")
        .unwrap()
        .bool(true)
        .unwrap();
    }

    let expected = Properties {
      gallery: vec![
        Item {
          id: Some(inscription_id(0)),
          attributes: Attributes {
            title: Some("bar".into()),
            traits: Traits::default(),
          },
          index: None,
        },
        Item {
          id: Some(inscription_id(1)),
          attributes: Attributes {
            title: Some("baz".into()),
            traits: Traits {
              items: vec![("abc".into(), Trait::String("xyz".into()))],
            },
          },
          index: None,
        },
        Item {
          id: Some(inscription_id(2)),
          attributes: Attributes::default(),
          index: None,
        },
      ],
      attributes: Attributes {
        title: Some("foo".into()),
        traits: Traits {
          items: vec![("hello".into(), Trait::Bool(true))],
        },
      },
      txids: Vec::new(),
    };

    assert_eq!(Properties::from_cbor(&buffer), expected);

    assert_eq!(expected.to_inline_cbor(), Some(buffer.clone()));
  }

  #[test]
  fn trait_names_may_not_be_duplicated() {
    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(1)
        .unwrap()
        .u8(1)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(1)
        .unwrap()
        .map(2)
        .unwrap()
        .str("foo")
        .unwrap()
        .null()
        .unwrap()
        .str("foo")
        .unwrap()
        .null()
        .unwrap();
    }

    assert_eq!(Properties::from_cbor(&buffer), Properties::default());
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

  #[test]
  fn trait_cbor_representation() {
    #[track_caller]
    fn case(value: Trait, cbor: &[u8]) {
      assert_eq!(minicbor::to_vec(value).unwrap(), cbor);
    }

    case(Trait::Bool(false), &[244]);
    case(Trait::Bool(true), &[245]);
    case(Trait::Null, &[246]);
    case(Trait::Integer(0), &[0]);
    case(Trait::Integer(1), &[1]);
    case(Trait::String("foo".into()), b"\x63foo");
  }

  #[test]
  fn trait_json_representation() {
    #[track_caller]
    fn case(value: Trait, json: &str) {
      assert_eq!(serde_json::to_string(&value).unwrap(), json);
    }

    case(Trait::Bool(false), "false");
    case(Trait::Bool(true), "true");
    case(Trait::Null, "null");
    case(Trait::Integer(0), "0");
    case(Trait::Integer(1), "1");
    case(Trait::String("foo".into()), "\"foo\"");
  }

  #[test]
  fn cbor_decode_errors() {
    use {
      minicbor::data::Token::{self, *},
      std::error::Error,
    };

    fn case<T: for<'a> minicbor::Decode<'a, ()>>(tokens: &[Token], error: &str) {
      let mut encoder = Encoder::new(Vec::new());

      encoder.tokens(tokens).unwrap();

      assert_eq!(
        minicbor::decode::<T>(&encoder.into_writer())
          .map(|_| ())
          .unwrap_err()
          .source()
          .unwrap()
          .to_string(),
        error,
      );
    }

    case::<Traits>(
      &[Map(2), String("foo"), Null, String("foo"), Null],
      "duplicate trait `foo`",
    );

    case::<Traits>(
      &[BeginMap, String("foo"), Null, Break],
      "indefinite length types are not allowed: indefinite map",
    );

    case::<Trait>(
      &[BeginString, Break],
      "indefinite length types are not allowed: indefinite string",
    );

    case::<Trait>(&[Bytes(&[])], "unexpected type: bytes");

    case::<Trait>(
      &[Int(
        minicbor::data::Int::try_from(-i128::from(u64::MAX)).unwrap(),
      )],
      "integer out of range",
    );

    case::<Trait>(&[Break], "unexpected break");

    case::<Item>(
      &[Map(1), U8(0), Bytes(&[])],
      "invalid inscription ID length 0",
    );
  }

  #[test]
  fn packed_round_trip() {
    let properties = Properties {
      gallery: vec![
        Item {
          id: Some(inscription_id(0)),
          attributes: Attributes {
            title: Some("foo".into()),
            ..default()
          },
          index: None,
        },
        Item {
          id: Some(inscription_id(1)),
          attributes: Attributes::default(),
          index: None,
        },
      ],
      attributes: Attributes::default(),
      txids: Vec::new(),
    };

    let packed = properties.to_packed_cbor().unwrap();
    assert_eq!(Properties::from_cbor(&packed), properties);
  }

  #[test]
  fn packed_non_zero_index() {
    let id = inscription_id(1);

    let properties = Properties {
      gallery: vec![Item {
        id: Some(id),
        attributes: Attributes::default(),
        index: None,
      }],
      attributes: Attributes::default(),
      txids: Vec::new(),
    };

    let packed = properties.to_packed_cbor().unwrap();
    assert_eq!(Properties::from_cbor(&packed), properties);
  }

  #[test]
  fn packed_shared_txids() {
    let txid = txid(0);

    let properties = Properties {
      gallery: vec![
        Item {
          id: Some(InscriptionId { txid, index: 0 }),
          attributes: Attributes::default(),
          index: None,
        },
        Item {
          id: Some(InscriptionId { txid, index: 1 }),
          attributes: Attributes::default(),
          index: None,
        },
        Item {
          id: Some(InscriptionId { txid, index: 2 }),
          attributes: Attributes::default(),
          index: None,
        },
      ],
      attributes: Attributes::default(),
      txids: Vec::new(),
    };

    let packed = properties.to_packed_cbor().unwrap();
    assert_eq!(Properties::from_cbor(&packed), properties);
  }

  #[test]
  fn packed_invalid_txids_length_produces_empty_gallery() {
    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(1)
        .unwrap()
        .map(0)
        .unwrap()
        .u8(2)
        .unwrap()
        .bytes(&[0; 31])
        .unwrap();
    }

    assert_eq!(
      Properties::from_cbor(&buffer),
      Properties {
        gallery: Vec::new(),
        attributes: Attributes::default(),
        txids: Vec::new(),
      },
    );
  }

  #[test]
  fn packed_empty_gallery() {
    let properties = Properties {
      gallery: Vec::new(),
      attributes: Attributes {
        title: Some("foo".into()),
        ..default()
      },
      txids: Vec::new(),
    };

    assert!(properties.to_packed_cbor().is_some());
  }

  #[test]
  fn fewer_packed_txids_with_inline_ids_is_valid() {
    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(2)
        .unwrap()
        .map(0)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(1).value())
        .unwrap()
        .u8(2)
        .unwrap()
        .bytes(&inscription_id(0).txid.to_byte_array())
        .unwrap();
    }

    assert_eq!(
      Properties::from_cbor(&buffer),
      Properties {
        gallery: vec![
          Item {
            id: Some(inscription_id(0)),
            attributes: Attributes::default(),
            index: None,
          },
          Item {
            id: Some(inscription_id(1)),
            attributes: Attributes::default(),
            index: None,
          },
        ],
        attributes: Attributes::default(),
        txids: Vec::new(),
      },
    );
  }

  #[test]
  fn fewer_packed_txids_without_inline_ids_clears_gallery() {
    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(2)
        .unwrap()
        .map(0)
        .unwrap()
        .map(0)
        .unwrap()
        .u8(2)
        .unwrap()
        .bytes(&inscription_id(0).txid.to_byte_array())
        .unwrap();
    }

    assert_eq!(
      Properties::from_cbor(&buffer),
      Properties {
        gallery: Vec::new(),
        attributes: Attributes::default(),
        txids: Vec::new(),
      },
    );
  }

  #[test]
  fn extra_packed_txids_are_ignored() {
    let mut txids = Vec::new();
    txids.extend_from_slice(&inscription_id(0).txid.to_byte_array());
    txids.extend_from_slice(&inscription_id(1).txid.to_byte_array());

    let mut buffer = Vec::new();

    {
      Encoder::new(&mut buffer)
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .array(1)
        .unwrap()
        .map(0)
        .unwrap()
        .u8(2)
        .unwrap()
        .bytes(&txids)
        .unwrap();
    }

    assert_eq!(
      Properties::from_cbor(&buffer),
      Properties {
        gallery: vec![Item {
          id: Some(inscription_id(0)),
          attributes: Attributes::default(),
          index: None,
        }],
        attributes: Attributes::default(),
        txids: Vec::new(),
      },
    );
  }

  #[test]
  fn packed_round_trip_with_attributes_and_non_zero_index() {
    let id = InscriptionId {
      txid: inscription_id(0).txid,
      index: 3,
    };

    let properties = Properties {
      gallery: vec![Item {
        id: Some(id),
        attributes: Attributes {
          title: Some("foo".into()),
          ..default()
        },
        index: None,
      }],
      attributes: Attributes::default(),
      txids: Vec::new(),
    };

    let packed = properties.to_packed_cbor().unwrap();
    assert_eq!(Properties::from_cbor(&packed), properties);
  }

  #[test]
  fn old_format_decode_still_works() {
    let properties = Properties {
      gallery: vec![Item {
        id: Some(inscription_id(0)),
        attributes: Attributes::default(),
        index: None,
      }],
      attributes: Attributes::default(),
      txids: Vec::new(),
    };

    let cbor = properties.to_inline_cbor().unwrap();
    assert_eq!(Properties::from_cbor(&cbor), properties);
  }
}
