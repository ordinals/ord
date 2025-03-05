use super::*;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Entry {
  pub delegate: Option<InscriptionId>,
  pub destination: Option<Address<NetworkUnchecked>>,
  pub file: Option<PathBuf>,
  #[serde(default)]
  pub gallery: Vec<InscriptionId>,
  pub metadata: Option<serde_yaml::Value>,
  pub metaprotocol: Option<String>,
  pub satpoint: Option<SatPoint>,
}

impl Entry {
  pub(crate) fn metadata(&self) -> Result<Option<Vec<u8>>> {
    match &self.metadata {
      None => Ok(None),
      Some(metadata) => {
        let mut cbor = Vec::new();
        ciborium::into_writer(&metadata, &mut cbor)?;
        Ok(Some(cbor))
      }
    }
  }

  pub(crate) fn properties(&self) -> Properties {
    Properties {
      gallery: self.gallery.clone(),
    }
  }
}
