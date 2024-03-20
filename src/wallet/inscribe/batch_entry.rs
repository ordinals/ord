use super::*;

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BatchEntry {
  pub delegate: Option<InscriptionId>,
  pub destination: Option<Address<NetworkUnchecked>>,
  pub file: PathBuf,
  pub metadata: Option<serde_yaml::Value>,
  pub metaprotocol: Option<String>,
  pub satpoint: Option<SatPoint>,
}

impl BatchEntry {
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
