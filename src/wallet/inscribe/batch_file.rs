use super::*;

#[derive(Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Batchfile {
  pub(crate) inscriptions: Vec<BatchEntry>,
  pub(crate) mode: Mode,
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) postage: Option<u64>,
  pub(crate) sat: Option<Sat>,
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
    wallet: &Wallet,
    parent_value: Option<u64>,
    metadata: Option<Vec<u8>>,
    postage: Amount,
    compress: bool,
  ) -> Result<(Vec<Inscription>, Vec<Address>)> {
    assert!(!self.inscriptions.is_empty());

    if self
      .inscriptions
      .iter()
      .any(|entry| entry.destination.is_some())
      && (self.mode == Mode::SharedOutput || self.mode == Mode::SameSat)
    {
      return Err(anyhow!(
        "individual inscription destinations cannot be set in shared-output or same-sat mode"
      ));
    }

    if metadata.is_some() {
      assert!(self
        .inscriptions
        .iter()
        .all(|entry| entry.metadata.is_none()));
    }

    let mut pointer = parent_value.unwrap_or_default();

    let mut inscriptions = Vec::new();
    for (i, entry) in self.inscriptions.iter().enumerate() {
      if let Some(delegate) = entry.delegate {
        ensure! {
          wallet.inscription_exists(delegate)?,
          "delegate {delegate} does not exist"
        }
      }

      inscriptions.push(Inscription::from_file(
        wallet.chain(),
        compress,
        entry.delegate,
        match &metadata {
          Some(metadata) => Some(metadata.clone()),
          None => entry.metadata()?,
        },
        entry.metaprotocol.clone(),
        self.parent,
        &entry.file,
        if i == 0 { None } else { Some(pointer) },
      )?);

      pointer += postage.to_sat();
    }

    let destinations = match self.mode {
      Mode::SharedOutput | Mode::SameSat => vec![wallet.get_change_address()?],
      Mode::SeparateOutputs => self
        .inscriptions
        .iter()
        .map(|entry| {
          entry.destination.as_ref().map_or_else(
            || wallet.get_change_address(),
            |address| {
              address
                .clone()
                .require_network(wallet.chain().network())
                .map_err(|e| e.into())
            },
          )
        })
        .collect::<Result<Vec<_>, _>>()?,
    };

    Ok((inscriptions, destinations))
  }
}
