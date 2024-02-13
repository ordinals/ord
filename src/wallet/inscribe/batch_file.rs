use super::*;

#[derive(Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Batchfile {
  pub(crate) inscriptions: Vec<BatchEntry>,
  pub(crate) mode: Mode,
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) postage: Option<u64>,
  pub(crate) sat: Option<Sat>,
  pub(crate) satpoint: Option<SatPoint>,
}

impl Batchfile {
  pub(crate) fn load(path: &Path) -> Result<Batchfile> {
    let batchfile: Batchfile = serde_yaml::from_reader(File::open(path)?)?;

    ensure!(
      !batchfile.inscriptions.is_empty(),
      "batchfile must contain at least one inscription",
    );

    let sat_and_satpoint = batchfile.sat.is_some() && batchfile.satpoint.is_some();

    ensure!(
      !sat_and_satpoint,
      "batchfile cannot set both `sat` and `satpoint`",
    );

    let sat_or_satpoint = batchfile.sat.is_some() || batchfile.satpoint.is_some();

    if sat_or_satpoint {
      ensure!(
        batchfile.mode == Mode::SameSat,
        "neither `sat` nor `satpoint` can be set in `same-sat` mode",
      );
    }

    if batchfile
      .inscriptions
      .iter()
      .any(|entry| entry.destination.is_some())
      && (batchfile.mode == Mode::SharedOutput || batchfile.mode == Mode::SameSat)
    {
      bail!(
        "individual inscription destinations cannot be set in `shared-output` or `same-sat` mode"
      );
    }

    let any_entry_has_satpoint = batchfile
      .inscriptions
      .iter()
      .any(|entry| entry.satpoint.is_some());

    if any_entry_has_satpoint {
      ensure!(
        batchfile.mode == Mode::SatPoints,
        "specifying `satpoint` in an inscription only works in `satpoints` mode"
      );

      ensure!(
        batchfile.inscriptions.iter().all(|entry| entry.satpoint.is_some()),
        "if `satpoint` is set for any inscription, then all inscriptions need to specify a satpoint"
      );

      ensure!(
        batchfile
          .inscriptions
          .iter()
          .all(|entry| entry.satpoint.unwrap().offset == 0),
        "`satpoint` can only be specified for first sat of an output"
      );

      ensure!(
        batchfile.postage.is_none(),
        "`postage` cannot be set if `satpoint` is set for any inscription"
      );
    }

    Ok(batchfile)
  }

  pub(crate) fn inscriptions(
    &self,
    wallet: &Wallet,
    utxos: &BTreeMap<OutPoint, Amount>,
    parent_value: Option<u64>,
    compress: bool,
  ) -> Result<(Vec<Inscription>, Vec<Amount>, Vec<Address>)> {
    let mut inscriptions = Vec::new();
    let mut pointer = parent_value.unwrap_or_default();
    let mut postages = Vec::new();

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
        entry.metadata()?,
        entry.metaprotocol.clone(),
        self.parent,
        &entry.file,
        if i == 0 { None } else { Some(pointer) },
      )?);

      pointer += if self.mode == Mode::SatPoints {
        let satpoint = entry
          .satpoint
          .ok_or_else(|| anyhow!("no satpoint specified for {}", entry.file.display()))?;

        utxos
          .get(&satpoint.outpoint)
          .ok_or_else(|| anyhow!("{} not in wallet", satpoint))?
          .to_sat()
      } else {
        self
          .postage
          .map(Amount::from_sat)
          .unwrap_or(TARGET_POSTAGE)
          .to_sat()
      };

      if self.mode == Mode::SameSat && i != 0 {
        continue;
      } else {
        postages.push(Amount::from_sat(pointer));
      }
    }

    let destinations = match self.mode {
      Mode::SharedOutput | Mode::SameSat => vec![wallet.get_change_address()?],
      Mode::SeparateOutputs | Mode::SatPoints => self
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

    Ok((inscriptions, postages, destinations))
  }
}
