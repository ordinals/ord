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
    let mut cbor = Vec::new();
    ciborium::into_writer(&self.metadata, &mut cbor)?;

    Ok(Some(cbor))
  }
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchConfig {
  pub(crate) mode: Mode,
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) batch: Vec<BatchEntry>,
}

impl BatchConfig {
  pub(crate) fn inscriptions(
    &self,
    chain: Chain,
    parent_info: Option<ParentInfo>,
  ) -> Result<(Vec<Inscription>, Amount)> {
    let mut pointer = if let Some(info) = parent_info.clone() {
      info.tx_out.value // Inscribe in first sat after parent output
    } else {
      0
    };

    let mut inscriptions = Vec::new();
    for entry in &self.batch {
      inscriptions.push(Inscription::from_file(
        chain,
        &entry.inscription,
        self.parent,
        Some(pointer),
        entry.metaprotocol.clone(),
        entry.metadata()?,
      )?);

      pointer += TransactionBuilder::TARGET_POSTAGE.to_sat();
    }

    Ok((inscriptions, Amount::from_sat(pointer)))
  }
}
