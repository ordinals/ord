use super::*;

#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchEntry {
  inscription: PathBuf,
  json_metadata: Option<PathBuf>,
  metaprotocol: Option<String>,
}

impl BatchEntry {
  fn metadata(&self) -> Result<Option<Vec<u8>>> {
    Inscribe::parse_metadata(None, self.json_metadata.clone())
  }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchConfig {
  batch: Vec<BatchEntry>,
  dry_run: bool,
  fee_rate: FeeRate,
  parent: Option<InscriptionId>,
  mode: Mode,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub reveal: Txid,
  pub total_fees: u64,
  pub parent: Option<InscriptionId>,
  pub inscriptions: Vec<InscriptionId>,
}

#[derive(Debug, Parser)]
pub(crate) struct BatchInscribe {
  #[arg(help = "Read YAML batch <FILE> that specifies all inscription info.")]
  pub(crate) file: PathBuf,
}

impl BatchInscribe {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let batch_config = self.load_batch_config()?;

    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let parent_info =
      Inscribe::get_parent_info(batch_config.parent, &index, &utxos, &client, &options)?;

    let mut pointer = if let Some(info) = parent_info.clone() {
      info.tx_out.value // Inscribe in first sat after parent output
    } else {
      0
    };

    let mut inscriptions = Vec::new();
    for entry in &batch_config.batch {
      inscriptions.push(Inscription::from_file(
        options.chain(),
        &entry.inscription,
        batch_config.parent,
        Some(pointer),
        entry.metaprotocol.clone(),
        entry.metadata()?,
      )?);

      pointer += TransactionBuilder::TARGET_POSTAGE.to_sat();
    }

    let wallet_inscriptions = index.get_inscriptions(utxos.clone())?;

    let commit_tx_change = [
      get_change_address(&client, &options)?,
      get_change_address(&client, &options)?,
    ];

    let reveal_tx_destination = get_change_address(&client, &options)?;

    let (commit_tx, reveal_tx, recovery_key_pair, total_fees) =
      BatchInscribe::create_batch_inscription_transactions(
        &batch_config,
        parent_info,
        &inscriptions,
        wallet_inscriptions,
        options.chain().network(),
        utxos,
        commit_tx_change,
        reveal_tx_destination,
        TransactionBuilder::TARGET_POSTAGE,
      )?;

    if batch_config.dry_run {
      return Ok(Box::new(Output {
        commit: commit_tx.txid(),
        reveal: reveal_tx.txid(),
        total_fees,
        parent: batch_config.parent,
        inscriptions: vec![],
      }));
    }

    let signed_commit_tx = client
      .sign_raw_transaction_with_wallet(&commit_tx, None, None)?
      .hex;

    let signed_reveal_tx = if batch_config.parent.is_some() {
      client
        .sign_raw_transaction_with_wallet(
          &reveal_tx,
          Some(
            &commit_tx
              .output
              .iter()
              .enumerate()
              .map(|(vout, output)| SignRawTransactionInput {
                txid: commit_tx.txid(),
                vout: vout.try_into().unwrap(),
                script_pub_key: output.script_pubkey.clone(),
                redeem_script: None,
                amount: Some(Amount::from_sat(output.value)),
              })
              .collect::<Vec<SignRawTransactionInput>>(),
          ),
          None,
        )?
        .hex
    } else {
      bitcoin::consensus::encode::serialize(&reveal_tx)
    };

    Inscribe::backup_recovery_key(&client, recovery_key_pair, options.chain().network())?;

    let commit = client.send_raw_transaction(&signed_commit_tx)?;

    let reveal = match client.send_raw_transaction(&signed_reveal_tx) {
      Ok(txid) => txid,
      Err(err) => {
        return Err(anyhow!(
        "Failed to send reveal transaction: {err}\nCommit tx {commit} will be recovered once mined"
      ))
      }
    };

    Ok(Box::new(Output {
      commit,
      reveal,
      total_fees,
      parent: batch_config.parent,
      inscriptions: inscriptions
        .iter()
        .enumerate()
        .map(|(index, _inscription)| InscriptionId {
          txid: reveal,
          index: index.try_into().unwrap(),
        })
        .collect(),
    }))
  }

  pub(crate) fn load_batch_config(&self) -> Result<BatchConfig> {
    Ok(serde_yaml::from_reader(File::open(self.file.clone())?)?)
  }

  fn create_batch_inscription_transactions(
    batch_config: &BatchConfig,
    parent_info: Option<ParentInfo>,
    inscriptions: &Vec<Inscription>,
    wallet_inscriptions: BTreeMap<SatPoint, InscriptionId>,
    network: Network,
    mut utxos: BTreeMap<OutPoint, Amount>,
    change: [Address; 2],
    destination: Address,
    postage: Amount,
  ) -> Result<(Transaction, Transaction, TweakedKeyPair, u64)> {
    let satpoint = {
      let inscribed_utxos = wallet_inscriptions
        .keys()
        .map(|satpoint| satpoint.outpoint)
        .collect::<BTreeSet<OutPoint>>();

      utxos
        .keys()
        .find(|outpoint| !inscribed_utxos.contains(outpoint))
        .map(|outpoint| SatPoint {
          outpoint: *outpoint,
          offset: 0,
        })
        .ok_or_else(|| anyhow!("wallet contains no cardinal utxos"))?
    };

    for (inscribed_satpoint, inscription_id) in &wallet_inscriptions {
      if *inscribed_satpoint == satpoint {
        return Err(anyhow!("sat at {} already inscribed", satpoint));
      }

      if inscribed_satpoint.outpoint == satpoint.outpoint {
        return Err(anyhow!(
          "utxo {} already inscribed with inscription {inscription_id} on sat {inscribed_satpoint}",
          satpoint.outpoint,
        ));
      }
    }

    let secp256k1 = Secp256k1::new();
    let key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let reveal_script = Inscription::append_batch_reveal_script(
      inscriptions,
      ScriptBuf::builder()
        .push_slice(public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG),
    );

    let taproot_spend_info = TaprootBuilder::new()
      .add_leaf(0, reveal_script.clone())
      .expect("adding leaf should work")
      .finalize(&secp256k1, public_key)
      .expect("finalizing taproot builder should work");

    let control_block = taproot_spend_info
      .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let commit_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);

    let mut inputs = vec![OutPoint::null()];
    let mut outputs = vec![TxOut {
      script_pubkey: destination.script_pubkey(),
      value: 0,
    }];

    if let Some(ParentInfo {
      location,
      destination,
      tx_out,
    }) = parent_info.clone()
    {
      inputs.insert(0, location.outpoint);
      outputs.insert(
        0,
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: tx_out.value,
        },
      );
    }

    let commit_input = if parent_info.is_some() { 1 } else { 0 };

    let (_, reveal_fee) = Inscribe::build_reveal_transaction(
      &control_block,
      batch_config.fee_rate,
      inputs.clone(),
      commit_input,
      outputs.clone(),
      &reveal_script,
    );

    let unsigned_commit_tx = TransactionBuilder::new(
      satpoint,
      wallet_inscriptions,
      utxos.clone(),
      commit_tx_address.clone(),
      change,
      batch_config.fee_rate,
      Target::Value(reveal_fee + postage),
    )
    .build_transaction()?;

    let (vout, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
      .expect("should find sat commit/inscription output");

    inputs[commit_input] = OutPoint {
      txid: unsigned_commit_tx.txid(),
      vout: vout.try_into().unwrap(),
    };

    outputs[commit_input] = TxOut {
      script_pubkey: destination.script_pubkey(),
      value: output.value,
    };

    let (mut reveal_tx, fee) = Inscribe::build_reveal_transaction(
      &control_block,
      batch_config.fee_rate,
      inputs,
      commit_input,
      outputs.clone(),
      &reveal_script,
    );

    reveal_tx.output[commit_input].value = reveal_tx.output[commit_input]
      .value
      .checked_sub(fee.to_sat())
      .context("commit transaction output value insufficient to pay transaction fee")?;

    if reveal_tx.output[commit_input].value
      < reveal_tx.output[commit_input]
        .script_pubkey
        .dust_value()
        .to_sat()
    {
      bail!("commit transaction output would be dust");
    }

    let mut prevouts = vec![unsigned_commit_tx.output[vout].clone()];

    if let Some(parent_info) = parent_info {
      prevouts.insert(0, parent_info.tx_out);
    }

    let mut sighash_cache = SighashCache::new(&mut reveal_tx);

    let sighash = sighash_cache
      .taproot_script_spend_signature_hash(
        commit_input,
        &Prevouts::All(&prevouts),
        TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript),
        TapSighashType::Default,
      )
      .expect("signature hash should compute");

    let sig = secp256k1.sign_schnorr(
      &secp256k1::Message::from_slice(sighash.as_ref())
        .expect("should be cryptographically secure hash"),
      &key_pair,
    );

    let witness = sighash_cache
      .witness_mut(commit_input)
      .expect("getting mutable witness reference should work");

    witness.push(
      Signature {
        sig,
        hash_ty: TapSighashType::Default,
      }
      .to_vec(),
    );

    witness.push(reveal_script);
    witness.push(&control_block.serialize());

    let recovery_key_pair = key_pair.tap_tweak(&secp256k1, taproot_spend_info.merkle_root());

    let (x_only_pub_key, _parity) = recovery_key_pair.to_inner().x_only_public_key();
    assert_eq!(
      Address::p2tr_tweaked(
        TweakedPublicKey::dangerous_assume_tweaked(x_only_pub_key),
        network,
      ),
      commit_tx_address
    );

    let reveal_weight = reveal_tx.weight();

    if reveal_weight > bitcoin::Weight::from_wu(MAX_STANDARD_TX_WEIGHT.into()) {
      bail!(
        "reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {reveal_weight}"
      );
    }

    utxos.insert(
      reveal_tx.input[commit_input].previous_output,
      Amount::from_sat(
        unsigned_commit_tx.output[reveal_tx.input[commit_input].previous_output.vout as usize]
          .value,
      ),
    );

    let total_fees = Inscribe::calculate_fee(&unsigned_commit_tx, &utxos)
      + Inscribe::calculate_fee(&reveal_tx, &utxos);

    Ok((unsigned_commit_tx, reveal_tx, recovery_key_pair, total_fees))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn batch_is_loaded_from_yaml_file() {
    let parent = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let tempdir = TempDir::new().unwrap();

    let inscription_path = tempdir.path().join("tulip.txt");
    let metadata_path = tempdir.path().join("metadata.json");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let brc20_path = tempdir.path().join("token.json");

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "dry_run: false\nfee_rate: 2.1\nmode: shared-output\nparent: {parent}\nbatch:\n- inscription: {}\n  json_metadata: {}\n- inscription: {}\n  metaprotocol: brc-20\n",
        inscription_path.display(),
        metadata_path.display(),
        brc20_path.display()
      ),
    )
    .unwrap();

    pretty_assert_eq!(
      match Arguments::try_parse_from([
        "ord",
        "wallet",
        "batch-inscribe",
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
        batch: vec![
          BatchEntry {
            inscription: inscription_path,
            json_metadata: Some(metadata_path),
            ..Default::default()
          },
          BatchEntry {
            inscription: brc20_path,
            metaprotocol: Some("brc-20".to_string()),
            ..Default::default()
          }
        ],
        dry_run: false,
        fee_rate: FeeRate::try_from(2.1).unwrap(),
        parent: Some(parent),
        mode: Mode::SharedOutput,
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
}
