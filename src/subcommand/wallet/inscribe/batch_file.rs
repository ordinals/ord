use super::*;

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum Mode {
  #[serde(rename = "separate-outputs")]
  SeparateOutputs,
  #[serde(rename = "shared-output")]
  SharedOutput,
}

#[derive(Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchEntry {
  pub(crate) file: PathBuf,
  pub(crate) metadata: Option<serde_yaml::Value>,
  pub(crate) metaprotocol: Option<String>,
  pub(crate) destination: Option<Address<NetworkUnchecked>>,
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
pub(crate) struct Batchfile {
  pub(crate) inscriptions: Vec<BatchEntry>,
  pub(crate) mode: Mode,
  pub(crate) parent: Option<InscriptionId>,
}

impl Batchfile {
  pub(crate) fn load(path: &Path) -> Result<Batchfile> {
    let batchfile: Batchfile = serde_yaml::from_reader(File::open(path)?)?;

    if batchfile.inscriptions.is_empty() {
      bail!("batchfile must contain at least one inscription");
    }

    Ok(batchfile)
  }

  pub(crate) fn inscriptions(
    &self,
    client: &Client,
    chain: Chain,
    parent_value: Option<u64>,
    metadata: Option<Vec<u8>>,
    postage: Amount,
  ) -> Result<(Vec<Inscription>, Vec<Address>)> {
    assert!(!self.inscriptions.is_empty());

    if metadata.is_some() {
      assert!(self
        .inscriptions
        .iter()
        .all(|entry| entry.metadata.is_none()));
    }

    let mut pointer = parent_value.unwrap_or_default();

    let mut inscriptions = Vec::new();
    let mut destinations = Vec::new();
    for (i, entry) in self.inscriptions.iter().enumerate() {
      inscriptions.push(Inscription::from_file(
        chain,
        &entry.file,
        self.parent,
        if i == 0 { None } else { Some(pointer) },
        entry.metaprotocol.clone(),
        match &metadata {
          Some(metadata) => Some(metadata.clone()),
          None => entry.metadata()?,
        },
      )?);

      if !(self.mode == Mode::SharedOutput && i >= 1) {
        destinations.push(entry.destination.as_ref().map_or_else(
          || get_change_address(client, chain),
          |address| {
            address
              .clone()
              .require_network(chain.network())
              .map_err(|e| e.into())
          },
        )?);
      }

      pointer += postage.to_sat();
    }

    Ok((inscriptions, destinations))
  }
}
