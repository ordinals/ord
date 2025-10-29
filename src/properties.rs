use super::*;

mod raw;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
  pub title: Option<String>,
}

impl Attributes {
  fn to_raw(&self) -> Option<raw::Attributes> {
    if *self == Default::default() {
      None
    } else {
      Some(raw::Attributes {
        title: self.title.clone(),
      })
    }
  }
}

impl From<raw::Attributes> for Attributes {
  fn from(raw: raw::Attributes) -> Self {
    Self { title: raw.title }
  }
}

#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
  pub id: InscriptionId,
  #[serde(flatten)]
  pub attributes: Attributes,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Properties {
  pub(crate) gallery: Vec<Item>,
  pub(crate) attributes: Attributes,
}

impl Properties {
  pub(crate) fn from_cbor(cbor: &[u8]) -> Self {
    let raw::Properties {
      gallery,
      attributes,
    } = minicbor::decode(cbor).unwrap_or_default();

    Self {
      gallery: gallery
        .filter(|gallery| gallery.iter().all(|item| item.id.is_some()))
        .unwrap_or_default()
        .into_iter()
        .map(|item| Item {
          id: item.id.unwrap(),
          attributes: item.attributes.unwrap_or_default().into(),
        })
        .collect(),
      attributes: attributes.unwrap_or_default().into(),
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
          },
        },
        Item {
          id: inscription_id(1),
          attributes: Attributes {
            title: Some("baz".into()),
          },
        },
      ],
      attributes: Attributes {
        title: Some("foo".into()),
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
