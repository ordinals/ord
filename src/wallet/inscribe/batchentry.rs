use super::*;

#[derive(Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BatchEntry {
  pub(crate) destination: Option<Address<NetworkUnchecked>>,
  pub(crate) file: PathBuf,
  pub(crate) metadata: Option<serde_yaml::Value>,
  pub(crate) metaprotocol: Option<String>,
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
