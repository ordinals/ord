use super::*;

pub use raw::{Trait, Traits};

mod raw;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
  pub title: Option<String>,
  #[serde(default)]
  pub traits: Traits,
}

impl Attributes {
  fn to_raw(&self) -> Option<raw::Attributes> {
    if *self == Default::default() {
      None
    } else {
      Some(raw::Attributes {
        title: self.title.clone(),
        traits: if self.traits.items.is_empty() {
          None
        } else {
          Some(self.traits.clone())
        },
      })
    }
  }
}

impl From<raw::Attributes> for Attributes {
  fn from(raw: raw::Attributes) -> Self {
    Self {
      title: raw.title,
      traits: raw.traits.unwrap_or_default(),
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
  pub attributes: Attributes,
  pub id: InscriptionId,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Properties {
  pub attributes: Attributes,
  pub gallery: Vec<Item>,
}

impl Properties {
  pub(crate) fn from_cbor(cbor: &[u8]) -> Self {
    let raw::Properties {
      attributes,
      gallery,
    } = minicbor::decode(cbor).unwrap_or_default();

    Self {
      attributes: attributes.unwrap_or_default().into(),
      gallery: gallery
        .filter(|gallery| gallery.iter().all(|item| item.id.is_some()))
        .unwrap_or_default()
        .into_iter()
        .map(|item| Item {
          id: item.id.unwrap(),
          attributes: item.attributes.unwrap_or_default().into(),
        })
        .collect(),
    }
  }

  pub(crate) fn to_cbor(&self) -> Option<Vec<u8>> {
    if *self == Self::default() {
      return None;
    }

    Some(
      minicbor::to_vec(raw::Properties {
        gallery: if self.gallery.is_empty() {
          None
        } else {
          Some(
            self
              .gallery
              .iter()
              .map(|item| raw::Item {
                id: Some(item.id),
                attributes: item.attributes.to_raw(),
              })
              .collect(),
          )
        },
        attributes: self.attributes.to_raw(),
      })
      .unwrap(),
    )
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
    assert_eq!(Properties::default().to_cbor(), None);

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
          id: inscription_id(0),
          attributes: Attributes {
            title: Some("bar".into()),
            traits: Traits::default(),
          },
        },
        Item {
          id: inscription_id(1),
          attributes: Attributes {
            title: Some("baz".into()),
            traits: Traits {
              items: vec![("abc".into(), Trait::String("xyz".into()))],
            },
          },
        },
        Item {
          id: inscription_id(2),
          attributes: Attributes::default(),
        },
      ],
      attributes: Attributes {
        title: Some("foo".into()),
        traits: Traits {
          items: vec![("hello".into(), Trait::Bool(true))],
        },
      },
    };

    assert_eq!(Properties::from_cbor(&buffer), expected);

    assert_eq!(expected.to_cbor(), Some(buffer.clone()));
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

    case::<raw::Item>(
      &[Map(1), U8(0), Bytes(&[])],
      "invalid inscription ID length 0",
    );
  }
}
