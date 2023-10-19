use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub reveal: Txid,
  pub total_fees: u64,
  pub parent: Option<InscriptionId>,
  pub inscriptions: Vec<InscriptionInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct InscriptionInfo {
  pub id: InscriptionId,
  pub location: SatPoint,
}

impl Display for InscriptionInfo {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "id: {}\nlocation: {}", self.id, self.location)
  }
}

#[derive(Debug, Parser)]
pub(crate) struct BatchInscribe {
  #[arg(long, help = "Don't sign or broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
  pub(crate) fee_rate: FeeRate,
  #[arg(help = "Read YAML batch <FILE> that specifies all inscription info.")]
  pub(crate) file: PathBuf,
}

impl BatchInscribe {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let batch_config = self.load_batch_config()?;

    Ok(Box::new(batch_config.inscribe(
      &options,
      None,
      self.fee_rate,
      self.dry_run,
      None,
      false,
      None,
      false,
    )?))
  }

  pub(crate) fn load_batch_config(&self) -> Result<BatchConfig> {
    Ok(serde_yaml::from_reader(File::open(self.file.clone())?)?)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    serde_yaml::{Mapping, Value},
  };

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

    pretty_assert_eq!(
      match Arguments::try_parse_from([
        "ord",
        "wallet",
        "batch-inscribe",
        "--fee-rate",
        "4.4",
        batch_path.to_str().unwrap(),
      ])
      .unwrap()
      .subcommand
      {
        Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
          batch_inscribe.load_batch_config().unwrap(),
        _ => panic!(),
      },
      BatchConfig {
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

    assert!(match Arguments::try_parse_from([
      "ord",
      "wallet",
      "batch-inscribe",
      "--fee-rate",
      "5.5",
      batch_path.to_str().unwrap(),
    ])
    .unwrap()
    .subcommand
    {
      Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
        batch_inscribe.load_batch_config().is_err(),
      _ => panic!(),
    })
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

    assert!(match Arguments::try_parse_from([
      "ord",
      "wallet",
      "batch-inscribe",
      "--fee-rate",
      "21",
      batch_path.to_str().unwrap(),
    ])
    .unwrap()
    .subcommand
    {
      Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
        batch_inscribe.load_batch_config().is_err(),
      _ => panic!(),
    })
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
      inscription("text/plain", [b'O'; 100]),
      inscription("text/plain", [b'O'; 111]),
      inscription("text/plain", [b'O'; 222]),
    ];

    let postage = Amount::from_sat(30_000);

    let mode = Mode::SharedOutput;

    let fee_rate = 4.0.try_into().unwrap();

    let (commit_tx, reveal_tx, _private_key, _) =
      BatchConfig::create_batch_inscription_transactions(
        Some(parent_info.clone()),
        &inscriptions,
        wallet_inscriptions,
        Chain::Signet,
        utxos.into_iter().collect(),
        [commit_address, change(2)],
        reveal_addresses,
        None,
        fee_rate,
        postage,
        mode,
        None,
        None,
        false,
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
      inscription("text/plain", [b'O'; 100]),
      inscription("text/plain", [b'O'; 111]),
      inscription("text/plain", [b'O'; 222]),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient()];

    let error = BatchConfig::create_batch_inscription_transactions(
      Some(parent_info.clone()),
      &inscriptions,
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      reveal_addresses,
      None,
      4.0.try_into().unwrap(),
      Amount::from_sat(30_000),
      Mode::SharedOutput,
      None,
      None,
      false,
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
      inscription("text/plain", [b'O'; 100]),
      inscription("text/plain", [b'O'; 111]),
      inscription("text/plain", [b'O'; 222]),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient(), recipient()];

    let _ = BatchConfig::create_batch_inscription_transactions(
      Some(parent_info.clone()),
      &inscriptions,
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      reveal_addresses,
      None,
      4.0.try_into().unwrap(),
      Amount::from_sat(30_000),
      Mode::SharedOutput,
      None,
      None,
      false,
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

    let error = BatchConfig::create_batch_inscription_transactions(
      None,
      &inscriptions,
      wallet_inscriptions,
      Chain::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      reveal_addresses,
      None,
      1.0.try_into().unwrap(),
      Amount::from_sat(30_000),
      Mode::SharedOutput,
      None,
      None,
      false,
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

    let (_commit_tx, reveal_tx, _private_key, _) =
      BatchConfig::create_batch_inscription_transactions(
        None,
        &inscriptions,
        wallet_inscriptions,
        Chain::Signet,
        utxos.into_iter().collect(),
        [commit_address, change(2)],
        reveal_addresses,
        None,
        fee_rate,
        total_postage,
        mode,
        None,
        None,
        false,
      )
      .unwrap();

    //let sig_vbytes = 17;
    //let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    //let reveal_value = commit_tx
    //  .output
    //  .iter()
    //  .map(|o| o.value)
    //  .reduce(|acc, i| acc + i)
    //  .unwrap();

    // let sig_vbytes = 17;
    // let fee = fee_rate.fee(reveal_tx.vsize() + sig_vbytes).to_sat();

    //assert_eq!(
    //  fee,
    //  commit_tx.output[0].value - reveal_tx.output.iter().map(|o| o.value).sum::<u64>()
    //);
    // assert_eq!(commit_tx.output[0].value, total_postage.to_sat() + fee);
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
      inscription("text/plain", [b'O'; 100]),
      inscription("text/plain", [b'O'; 111]),
      inscription("text/plain", [b'O'; 222]),
    ];

    let postage = Amount::from_sat(30_000);

    let mode = Mode::SeparateOutputs;

    let fee_rate = 4.0.try_into().unwrap();

    let (_commit_tx, reveal_tx, _private_key, _) =
      BatchConfig::create_batch_inscription_transactions(
        Some(parent_info.clone()),
        &inscriptions,
        wallet_inscriptions,
        Chain::Signet,
        utxos.into_iter().collect(),
        [commit_address, change(2)],
        reveal_addresses,
        None,
        fee_rate,
        postage,
        mode,
        None,
        None,
        false,
      )
      .unwrap();

    //let sig_vbytes = 17;
    //let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    //let reveal_value = commit_tx
    //  .output
    //  .iter()
    //  .map(|o| o.value)
    //  .reduce(|acc, i| acc + i)
    //  .unwrap();

    //assert_eq!(reveal_value, 50_000 - fee);

    //let sig_vbytes = 16;
    //let fee = fee_rate.fee(reveal_tx.vsize() + sig_vbytes).to_sat();

    //assert_eq!(fee, commit_tx.output[0].value - reveal_tx.output[1].value,);
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
