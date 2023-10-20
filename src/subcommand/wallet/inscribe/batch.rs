use super::*;

#[derive(Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchEntry {
  pub(crate) inscription: PathBuf,
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

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchConfig {
  pub(crate) postage: Option<u64>,
  pub(crate) mode: Mode,
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) batch: Vec<BatchEntry>,
}

impl BatchConfig {
  pub(crate) fn load(path: &Path) -> Result<BatchConfig> {
    Ok(serde_yaml::from_reader(File::open(path)?)?)
  }

  pub(crate) fn inscriptions(
    &self,
    chain: Chain,
    parent_value: Option<u64>,
    metadata: Option<Vec<u8>>,
  ) -> Result<(Vec<Inscription>, Amount)> {
    if metadata.is_some() {
      assert!(!self.batch.iter().any(|entry| entry.metadata.is_some()));
    }

    let mut pointer = parent_value.unwrap_or_default();

    let mut inscriptions = Vec::new();
    for (i, entry) in self.batch.iter().enumerate() {
      inscriptions.push(Inscription::from_file(
        chain,
        &entry.inscription,
        self.parent,
        if i == 0 { None } else { Some(pointer) },
        entry.metaprotocol.clone(),
        match &metadata {
          Some(metadata) => Some(metadata.clone()),
          None => entry.metadata()?,
        },
      )?);

      pointer += self
        .postage
        .unwrap_or(TransactionBuilder::TARGET_POSTAGE.to_sat());
    }

    let total_postage = u64::try_from(inscriptions.len()).unwrap()
      * self
        .postage
        .unwrap_or(TransactionBuilder::TARGET_POSTAGE.to_sat());

    Ok((inscriptions, Amount::from_sat(total_postage)))
  }
}
