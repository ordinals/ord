use super::*;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Entry {
  pub file: Option<PathBuf>,
  pub delegate: Option<InscriptionId>,
  pub satpoint: Option<SatPoint>,
  pub destination: Option<Address<NetworkUnchecked>>,
  pub metadata: Option<serde_yaml::Value>,
  pub metaprotocol: Option<String>,
}

impl Entry {
  pub(crate) fn metadata(&self) -> Result<Option<Vec<u8>>> {
    Ok(match &self.metadata {
      None => None,
      Some(metadata) => {
        let mut cbor = Vec::new();
        ciborium::into_writer(&metadata, &mut cbor)?;
        Some(cbor)
      }
    })
  }
}
