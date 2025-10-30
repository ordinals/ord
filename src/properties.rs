use super::*;

pub use raw::{Trait, Traits};

mod raw;

// todo:
// - test
//   - cbor representation
//   - json representation
// - document
//
// later:
// - add keys, values, indices to batchfile
// - reconstruct indices, keys, values in API responses
// - top level indices using ord wallet batch
// - complain if two few or too many indices are passed
// - complain if indices are out of bounds
// - more trait types

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
        .array(2)
        .unwrap()
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
        .map(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .bytes(&inscription_id(1).value())
        .unwrap()
        .u8(1)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .str("baz")
        .unwrap()
        .u8(1)
        .unwrap()
        .map(1)
        .unwrap()
        .u8(0)
        .unwrap()
        .str("foo")
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
            traits: Traits::default(),
          },
        },
      ],
      attributes: Attributes {
        title: Some("foo".into()),
        traits: Traits::default(),
      },
    };

    assert_eq!(Properties::from_cbor(&buffer), expected);

    assert_eq!(expected.to_cbor(), Some(buffer.clone()));
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
