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

  // fn create_inscription_transactions(
  //   satpoint: Option<SatPoint>,
  //   parent_info: Option<ParentInfo>,
  //   inscriptions: Vec<Inscription>,
  //   wallet_inscriptions: BTreeMap<SatPoint, InscriptionId>,
  //   chain: Chain,
  //   utxos: BTreeMap<OutPoint, Amount>,
  //   change: [Address; 2],
  //   destinations: Vec<Address>,
  //   commit_fee_rate: FeeRate,
  //   reveal_fee_rate: FeeRate,
  //   no_limit: bool,
  //   reinscribe: bool,
  //   postage: Amount,
  //   total_postage: Amount,
  //   mode: Mode,
  // ) -> Result<(Transaction, Transaction, TweakedKeyPair, u64)> {

  pub(crate) fn create_batch_inscription_transactions(
    parent_info: Option<ParentInfo>,
    inscriptions: Vec<Inscription>,
    wallet_inscriptions: BTreeMap<SatPoint, InscriptionId>,
    chain: Chain,
    utxos: BTreeMap<OutPoint, Amount>,
    change: [Address; 2],
    destinations: Vec<Address>,
    commit_fee_rate: Option<FeeRate>,
    fee_rate: FeeRate,
    total_postage: Amount,
    batch_mode: Mode,
    postage: Option<u64>,
    satpoint: Option<SatPoint>,
    reinscribe: bool,
    no_limit: bool,
  ) -> Result<(Transaction, Transaction, TweakedKeyPair, u64)> {
    Inscribe::create_inscription_transactions(
      satpoint,
      parent_info,
      inscriptions,
      wallet_inscriptions,
      chain,
      utxos,
      change,
      destinations,
      commit_fee_rate.unwrap_or(fee_rate),
      fee_rate,
      no_limit,
      reinscribe,
      postage
        .map(Amount::from_sat)
        .unwrap_or(TransactionBuilder::TARGET_POSTAGE),
      total_postage,
      batch_mode,
    )
  }
}
