use {
  self::batch::{Batch, Batchfile, Mode},
  super::*,
  crate::{subcommand::wallet::transaction_builder::Target, wallet::Wallet},
  bitcoin::{
    blockdata::{opcodes, script},
    key::PrivateKey,
    key::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
    locktime::absolute::LockTime,
    policy::MAX_STANDARD_TX_WEIGHT,
    secp256k1::{self, constants::SCHNORR_SIGNATURE_SIZE, rand, Secp256k1, XOnlyPublicKey},
    sighash::{Prevouts, SighashCache, TapSighashType},
    taproot::Signature,
    taproot::{ControlBlock, LeafVersion, TapLeafHash, TaprootBuilder},
    ScriptBuf, Witness,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, SignRawTransactionInput, Timestamp},
  bitcoincore_rpc::Client,
  std::collections::BTreeSet,
};

mod batch;

#[derive(Serialize, Deserialize)]
pub struct InscriptionInfo {
  pub id: InscriptionId,
  pub location: SatPoint,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub inscriptions: Vec<InscriptionInfo>,
  pub parent: Option<InscriptionId>,
  pub reveal: Txid,
  pub total_fees: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ParentInfo {
  destination: Address,
  id: InscriptionId,
  location: SatPoint,
  tx_out: TxOut,
}

#[derive(Debug, Parser)]
pub(crate) struct Inscribe {
  #[arg(
    long,
    help = "Inscribe a multiple inscriptions defines in a yaml <BATCH_FILE>.",
    conflicts_with_all = &["file", "destination", "cbor_metadata", "json_metadata", "commit_fee_rate", "no_backup", "satpoint", "reinscribe", "postage", "metaprotocol", "parent", "no_limit"]
  )]
  pub(crate) batch: Option<PathBuf>,
  #[arg(
    long,
    help = "Include CBOR in file at <METADATA> as inscription metadata",
    conflicts_with = "json_metadata"
  )]
  pub(crate) cbor_metadata: Option<PathBuf>,
  #[arg(
    long,
    help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
  )]
  pub(crate) commit_fee_rate: Option<FeeRate>,
  #[arg(long, help = "Send inscription to <DESTINATION>.")]
  pub(crate) destination: Option<Address<NetworkUnchecked>>,
  #[arg(long, help = "Don't sign or broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
  pub(crate) fee_rate: FeeRate,
  #[arg(help = "Inscribe sat with contents of <FILE>.")]
  pub(crate) file: Option<PathBuf>,
  #[arg(
    long,
    help = "Include JSON in file at <METADATA> convered to CBOR as inscription metadata",
    conflicts_with = "cbor_metadata"
  )]
  pub(crate) json_metadata: Option<PathBuf>,
  #[clap(long, help = "Set inscription metaprotocol to <METAPROTOCOL>.")]
  pub(crate) metaprotocol: Option<String>,
  #[arg(long, help = "Do not back up recovery key.")]
  pub(crate) no_backup: bool,
  #[arg(
    long,
    help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
  )]
  pub(crate) no_limit: bool,
  #[clap(long, help = "Make inscription a child of <PARENT>.")]
  pub(crate) parent: Option<InscriptionId>,
  #[arg(
    long,
    help = "Amount of postage to include in the inscription. Default `10000sat`."
  )]
  pub(crate) postage: Option<Amount>,
  #[clap(long, help = "Allow reinscription.")]
  pub(crate) reinscribe: bool,
  #[arg(long, help = "Inscribe <SATPOINT>.")]
  pub(crate) satpoint: Option<SatPoint>,
}

impl Inscribe {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let metadata = Inscribe::parse_metadata(self.cbor_metadata, self.json_metadata)?;

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let chain = options.chain();

    let inscriptions;
    let mode;
    let postage;
    let total_postage;
    let parent_info;
    let destinations;

    if let Some(batch) = self.batch {
      let batch_config = Batchfile::load(&batch)?;

      parent_info = Inscribe::get_parent_info(batch_config.parent, &index, &utxos, &client, chain)?;

      (inscriptions, total_postage) = batch_config.inscriptions(
        chain,
        parent_info.as_ref().map(|info| info.tx_out.value),
        metadata,
      )?;

      postage = batch_config
        .postage
        .map(Amount::from_sat)
        .unwrap_or(TransactionBuilder::TARGET_POSTAGE);

      mode = batch_config.mode;

      assert!(self.destination.is_none());

      let destination_count = match batch_config.mode {
        Mode::SharedOutput => 1,
        Mode::SeparateOutputs => inscriptions.len(),
      };

      destinations = (0..destination_count)
        .map(|_| get_change_address(&client, chain))
        .collect::<Result<Vec<Address>>>()?;
    } else {
      parent_info = Inscribe::get_parent_info(self.parent, &index, &utxos, &client, chain)?;
      inscriptions = vec![Inscription::from_file(
        chain,
        self.file.clone().unwrap(),
        self.parent,
        None,
        self.metaprotocol.clone(),
        metadata.clone(),
      )?];
      mode = Mode::SeparateOutputs;
      postage = self.postage.unwrap_or(TransactionBuilder::TARGET_POSTAGE);
      total_postage = postage;
      destinations = vec![match self.destination.clone() {
        Some(destination) => destination.require_network(chain.network())?,
        None => get_change_address(&client, chain)?,
      }];
    }

    Batch {
      commit_fee_rate: self.commit_fee_rate.unwrap_or(self.fee_rate),
      destinations,
      dry_run: self.dry_run,
      inscriptions,
      mode,
      no_backup: self.no_backup,
      no_limit: self.no_limit,
      parent_info,
      postage,
      reinscribe: self.reinscribe,
      reveal_fee_rate: self.fee_rate,
      satpoint: self.satpoint,
      total_postage,
    }
    .inscribe(chain, &index, &client, &utxos)
  }

  fn parse_metadata(cbor: Option<PathBuf>, json: Option<PathBuf>) -> Result<Option<Vec<u8>>> {
    if let Some(path) = cbor {
      let cbor = fs::read(path)?;
      let _value: Value = ciborium::from_reader(Cursor::new(cbor.clone()))
        .context("failed to parse CBOR metadata")?;

      Ok(Some(cbor))
    } else if let Some(path) = json {
      let value: serde_json::Value =
        serde_json::from_reader(File::open(path)?).context("failed to parse JSON metadata")?;
      let mut cbor = Vec::new();
      ciborium::into_writer(&value, &mut cbor)?;

      Ok(Some(cbor))
    } else {
      Ok(None)
    }
  }

  fn get_parent_info(
    parent: Option<InscriptionId>,
    index: &Index,
    utxos: &BTreeMap<OutPoint, Amount>,
    client: &Client,
    chain: Chain,
  ) -> Result<Option<ParentInfo>> {
    if let Some(parent_id) = parent {
      if let Some(satpoint) = index.get_inscription_satpoint_by_id(parent_id)? {
        if !utxos.contains_key(&satpoint.outpoint) {
          return Err(anyhow!(format!("parent {parent_id} not in wallet")));
        }

        Ok(Some(ParentInfo {
          destination: get_change_address(client, chain)?,
          id: parent_id,
          location: satpoint,
          tx_out: index
            .get_transaction(satpoint.outpoint.txid)?
            .expect("parent transaction not found in index")
            .output
            .into_iter()
            .nth(satpoint.outpoint.vout.try_into().unwrap())
            .expect("current transaction output"),
        }))
      } else {
        Err(anyhow!(format!("parent {parent_id} does not exist")))
      }
    } else {
      Ok(None)
    }
  }

  fn build_reveal_transaction(
    control_block: &ControlBlock,
    fee_rate: FeeRate,
    inputs: Vec<OutPoint>,
    commit_input_index: usize,
    outputs: Vec<TxOut>,
    script: &Script,
  ) -> (Transaction, Amount) {
    let reveal_tx = Transaction {
      input: inputs
        .iter()
        .map(|outpoint| TxIn {
          previous_output: *outpoint,
          script_sig: script::Builder::new().into_script(),
          witness: Witness::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        })
        .collect(),
      output: outputs,
      lock_time: LockTime::ZERO,
      version: 1,
    };

    let fee = {
      let mut reveal_tx = reveal_tx.clone();

      for (current_index, txin) in reveal_tx.input.iter_mut().enumerate() {
        // add dummy inscription witness for reveal input/commit output
        if current_index == commit_input_index {
          txin.witness.push(
            Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
              .unwrap()
              .to_vec(),
          );
          txin.witness.push(script);
          txin.witness.push(&control_block.serialize());
        } else {
          txin.witness = Witness::from_slice(&[&[0; SCHNORR_SIGNATURE_SIZE]]);
        }
      }

      fee_rate.fee(reveal_tx.vsize())
    };

    (reveal_tx, fee)
  }

  fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, Amount>) -> u64 {
    tx.input
      .iter()
      .map(|txin| utxos.get(&txin.previous_output).unwrap().to_sat())
      .sum::<u64>()
      .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<u64>())
      .unwrap()
  }

  fn backup_recovery_key(
    client: &Client,
    recovery_key_pair: TweakedKeyPair,
    network: Network,
  ) -> Result {
    let recovery_private_key = PrivateKey::new(recovery_key_pair.to_inner().secret_key(), network);

    let info = client.get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_wif()))?;

    let response = client.import_descriptors(ImportDescriptors {
      descriptor: format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
      timestamp: Timestamp::Now,
      active: Some(false),
      range: None,
      next_index: None,
      internal: Some(false),
      label: Some("commit tx recovery key".to_string()),
    })?;

    for result in response {
      if !result.success {
        return Err(anyhow!("commit tx recovery key import failed"));
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {
    self::batch::BatchEntry,
    super::*,
    serde_yaml::{Mapping, Value},
  };

  #[test]
  fn reveal_transaction_pays_fee() {
    let utxos = vec![(outpoint(1), Amount::from_sat(20000))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient();
    let change = [commit_address, change(1)];

    let (commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint: Some(satpoint(1, 0)),
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      utxos.into_iter().collect(),
      change,
    )
    .unwrap();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let fee = Amount::from_sat((1.0 * (reveal_tx.vsize() as f64)).ceil() as u64);

    assert_eq!(
      reveal_tx.output[0].value,
      20000 - fee.to_sat() - (20000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn inscribe_tansactions_opt_in_to_rbf() {
    let utxos = vec![(outpoint(1), Amount::from_sat(20000))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient();
    let change = [commit_address, change(1)];

    let (commit_tx, reveal_tx, _, _) = Batch {
      satpoint: Some(satpoint(1, 0)),
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      utxos.into_iter().collect(),
      change,
    )
    .unwrap();

    assert!(commit_tx.is_explicitly_rbf());
    assert!(reveal_tx.is_explicitly_rbf());
  }

  #[test]
  fn inscribe_with_no_satpoint_and_no_cardinal_utxos() {
    let utxos = vec![(outpoint(1), Amount::from_sat(1000))];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();

    let error = Batch {
      satpoint,
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      inscriptions,
      Chain::Mainnet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains("wallet contains no cardinal utxos"),
      "{}",
      error
    );
  }

  #[test]
  fn inscribe_with_no_satpoint_and_enough_cardinal_utxos() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(20_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();

    assert!(Batch {
      satpoint,
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      inscriptions,
      Chain::Mainnet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
    )
    .is_ok())
  }

  #[test]
  fn inscribe_with_custom_fee_rate() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let fee_rate = 3.3;

    let (commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint,
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      reveal_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + sig_vbytes)
      .to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value, 20_000 - fee);

    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize())
      .to_sat();

    assert_eq!(
      reveal_tx.output[0].value,
      20_000 - fee - (20_000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn inscribe_with_parent() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];

    let mut inscriptions = BTreeMap::new();
    let parent_inscription = inscription_id(1);
    let parent_info = ParentInfo {
      destination: change(3),
      id: parent_inscription,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: 10000,
      },
    };

    inscriptions.insert(parent_info.location, parent_inscription);

    let child_inscription = InscriptionTemplate {
      parent: Some(parent_inscription),
    }
    .into();

    let commit_address = change(1);
    let reveal_address = recipient();
    let fee_rate = 4.0;

    let (commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint: None,
      parent_info: Some(parent_info.clone()),
      inscriptions: vec![child_inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      reveal_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + sig_vbytes)
      .to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value, 20_000 - fee);

    let sig_vbytes = 16;
    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize() + sig_vbytes)
      .to_sat();

    assert_eq!(fee, commit_tx.output[0].value - reveal_tx.output[1].value,);
    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..Default::default()
      }
    );
  }

  #[test]
  fn inscribe_with_commit_fee_rate() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let commit_fee_rate = 3.3;
    let fee_rate = 1.0;

    let (commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint,
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(commit_fee_rate).unwrap(),
      reveal_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = FeeRate::try_from(commit_fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + sig_vbytes)
      .to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value, 20_000 - fee);

    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize())
      .to_sat();

    assert_eq!(
      reveal_tx.output[0].value,
      20_000 - fee - (20_000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn inscribe_over_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), Amount::from_sat(50 * COIN_VALUE))];

    let inscription = inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize]);
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();

    let error = Batch {
      satpoint,
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains(&format!("reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): 402799")),
      "{}",
      error
    );
  }

  #[test]
  fn inscribe_with_no_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), Amount::from_sat(50 * COIN_VALUE))];

    let inscription = inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize]);
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();

    let (_commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint,
      parent_info: None,
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: true,
      reinscribe: false,
      postage: TransactionBuilder::TARGET_POSTAGE,
      total_postage: TransactionBuilder::TARGET_POSTAGE,
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
    )
    .unwrap();

    assert!(reveal_tx.size() >= MAX_STANDARD_TX_WEIGHT as usize);
  }

  #[test]
  fn cbor_and_json_metadata_flags_conflict() {
    assert_regex_match!(
      Arguments::try_parse_from([
        "ord",
        "wallet",
        "inscribe",
        "--cbor-metadata",
        "foo",
        "--json-metadata",
        "bar",
        "baz",
      ])
      .unwrap_err()
      .to_string(),
      ".*--cbor-metadata.*cannot be used with.*--json-metadata.*"
    );
  }

  #[test]
  fn batch_is_loaded_from_yaml_file() {
    let parent = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let tempdir = TempDir::new().unwrap();

    let inscription_path = tempdir.path().join("tulip.txt");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let brc20_path = tempdir.path().join("token.json");

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "mode: separate-outputs
parent: {parent}
batch:
- inscription: {}
  metadata:
    title: Lorem Ipsum
    description: Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tristique, massa nec condimentum venenatis, ante massa tempor velit, et accumsan ipsum ligula a massa. Nunc quis orci ante.
- inscription: {}
  metaprotocol: brc-20
",
        inscription_path.display(),
        brc20_path.display()
      ),
    )
    .unwrap();

    let mut metadata = Mapping::new();
    metadata.insert(
      Value::String("title".to_string()),
      Value::String("Lorem Ipsum".to_string()),
    );
    metadata.insert(Value::String("description".to_string()), Value::String("Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tristique, massa nec condimentum venenatis, ante massa tempor velit, et accumsan ipsum ligula a massa. Nunc quis orci ante.".to_string()));

    assert_eq!(
      Batchfile::load(&batch_path).unwrap(),
      Batchfile {
        postage: None,
        batch: vec![
          BatchEntry {
            inscription: inscription_path,
            metadata: Some(Value::Mapping(metadata)),
            ..Default::default()
          },
          BatchEntry {
            inscription: brc20_path,
            metaprotocol: Some("brc-20".to_string()),
            ..Default::default()
          }
        ],
        parent: Some(parent),
        mode: Mode::SeparateOutputs,
      }
    );
  }

  #[test]
  fn batch_with_invalid_field_value_throws_error() {
    let tempdir = TempDir::new().unwrap();

    let inscription_path = tempdir.path().join("tulip.txt");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "mode: wrong-mode\nbatch:\n- inscription: {}\n",
        inscription_path.display(),
      ),
    )
    .unwrap();

    assert!(Batchfile::load(&batch_path).is_err());
  }

  #[test]
  fn batch_is_unknown_field_throws_error() {
    let tempdir = TempDir::new().unwrap();
    let inscription_path = tempdir.path().join("tulip.txt");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "mode: shared-output\nbatch:\n- inscription: {}\nunknown: 1.)what",
        inscription_path.display(),
      ),
    )
    .unwrap();

    assert!(Batchfile::load(&batch_path).is_err());
  }

  #[test]
  fn batch_inscribe_with_parent() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(50_000)),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: 10000,
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, parent);

    let commit_address = change(1);
    let reveal_addresses = vec![recipient()];

    let inscriptions = vec![
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
    ];

    let postage = Amount::from_sat(30_000);

    let mode = Mode::SharedOutput;

    let fee_rate = 4.0.try_into().unwrap();

    let (commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint: None,
      parent_info: Some(parent_info.clone()),
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      no_limit: false,
      reinscribe: false,
      postage: Amount::from_sat(10_000),
      total_postage: postage,
      mode,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value, 50_000 - fee);

    let sig_vbytes = 16;
    let fee = fee_rate.fee(reveal_tx.vsize() + sig_vbytes).to_sat();

    assert_eq!(fee, commit_tx.output[0].value - reveal_tx.output[1].value,);
    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..Default::default()
      }
    );
  }

  #[test]
  fn batch_inscribe_with_parent_not_enough_cardinals_utxos_fails() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: 10000,
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, parent);

    let inscriptions = vec![
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient()];

    let error = Batch {
      satpoint: None,
      parent_info: Some(parent_info.clone()),
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: 4.0.try_into().unwrap(),
      reveal_fee_rate: 4.0.try_into().unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: Amount::from_sat(10_000),
      total_postage: Amount::from_sat(30_000),
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains(
      "wallet does not contain enough cardinal UTXOs, please add additional funds to wallet."
    ));
  }

  #[test]
  #[should_panic(
    expected = "invariant: destination addresses and number of inscriptions doesn't match"
  )]
  fn batch_inscribe_with_inconsistent_reveal_addreses_panics() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(80_000)),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: 10000,
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, parent);

    let inscriptions = vec![
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient(), recipient()];

    let _ = Batch {
      satpoint: None,
      parent_info: Some(parent_info.clone()),
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: 4.0.try_into().unwrap(),
      reveal_fee_rate: 4.0.try_into().unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: Amount::from_sat(10_000),
      total_postage: Amount::from_sat(30_000),
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    );
  }

  #[test]
  fn batch_inscribe_over_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), Amount::from_sat(50 * COIN_VALUE))];

    let wallet_inscriptions = BTreeMap::new();

    let inscriptions = vec![
      inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize / 3]),
      inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize / 3]),
      inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize / 3]),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient()];

    let error = Batch {
      satpoint: None,
      parent_info: None,
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: 1.0.try_into().unwrap(),
      reveal_fee_rate: 1.0.try_into().unwrap(),
      no_limit: false,
      reinscribe: false,
      postage: Amount::from_sat(30_000),
      total_postage: Amount::from_sat(30_000),
      mode: Mode::SharedOutput,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains(&format!("reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): 402841")),
      "{}",
      error
    );
  }

  #[test]
  fn batch_inscribe_into_separate_outputs() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(80_000)),
    ];

    let wallet_inscriptions = BTreeMap::new();

    let commit_address = change(1);
    let reveal_addresses = vec![recipient(), recipient(), recipient()];

    let inscriptions = vec![
      inscription("text/plain", [b'O'; 100]),
      inscription("text/plain", [b'O'; 111]),
      inscription("text/plain", [b'O'; 222]),
    ];

    let mode = Mode::SeparateOutputs;
    let total_postage = Amount::from_sat(30_000);

    let fee_rate = 4.0.try_into().unwrap();

    let (_commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint: None,
      parent_info: None,
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      no_limit: false,
      reinscribe: false,
      postage: Amount::from_sat(10_000),
      total_postage,
      mode,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    )
    .unwrap();

    assert_eq!(reveal_tx.output.len(), 3);
    assert!(reveal_tx
      .output
      .iter()
      .all(|output| output.value == TransactionBuilder::TARGET_POSTAGE.to_sat()));
  }

  #[test]
  fn batch_inscribe_into_separate_outputs_with_parent() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(50_000)),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: 10000,
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, parent);

    let commit_address = change(1);
    let reveal_addresses = vec![recipient(), recipient(), recipient()];

    let inscriptions = vec![
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
      InscriptionTemplate {
        parent: Some(parent),
      }
      .into(),
    ];

    let postage = Amount::from_sat(30_000);

    let mode = Mode::SeparateOutputs;

    let fee_rate = 4.0.try_into().unwrap();

    let (commit_tx, reveal_tx, _private_key, _) = Batch {
      satpoint: None,
      parent_info: Some(parent_info.clone()),
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      no_limit: false,
      reinscribe: false,
      postage: Amount::from_sat(10_000),
      total_postage: postage,
      mode,
      ..Default::default()
    }
    .create_batch_inscription_transactions(
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
    )
    .unwrap();

    assert_eq!(
      parent,
      ParsedEnvelope::from_transaction(&reveal_tx)[0]
        .payload
        .parent()
        .unwrap()
    );
    assert_eq!(
      parent,
      ParsedEnvelope::from_transaction(&reveal_tx)[1]
        .payload
        .parent()
        .unwrap()
    );

    let sig_vbytes = 17;
    let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value, 50_000 - fee);

    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..Default::default()
      }
    );
  }
}
