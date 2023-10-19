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

  pub(crate) fn inscribe(
    &self,
    options: &Options,
    fee_rate: FeeRate,
    dry_run: bool,
  ) -> Result<crate::subcommand::wallet::inscribe::batch_inscribe::Output> {
    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let parent_info = Inscribe::get_parent_info(self.parent, &index, &utxos, &client, &options)?;

    let wallet_inscriptions = index.get_inscriptions(utxos.clone())?;

    let commit_tx_change = [
      get_change_address(&client, &options)?,
      get_change_address(&client, &options)?,
    ];

    let (inscriptions, postage) = self.inscriptions(options.chain(), parent_info.clone())?;

    let reveal_tx_destinations = match self.mode {
      Mode::SharedOutput => vec![get_change_address(&client, &options)?],
      Mode::SeparateOutputs => {
        let mut addresses = Vec::new();
        for _i in 0..inscriptions.len() {
          addresses.push(get_change_address(&client, &options)?)
        }

        addresses
      }
    };

    let (commit_tx, reveal_tx, recovery_key_pair, total_fees) =
      Self::create_batch_inscription_transactions(
        parent_info,
        &inscriptions,
        wallet_inscriptions,
        options.chain(),
        utxos,
        commit_tx_change,
        reveal_tx_destinations,
        fee_rate,
        postage,
        self.mode.clone(),
      )?;

    if dry_run {
      return Ok(Self::output(
        commit_tx.txid(),
        reveal_tx.txid(),
        self.clone(),
        total_fees,
        inscriptions,
      ));
    }

    let signed_commit_tx = client
      .sign_raw_transaction_with_wallet(&commit_tx, None, None)?
      .hex;

    let signed_reveal_tx = if self.parent.is_some() {
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

    Ok(Self::output(
      commit,
      reveal,
      self.clone(),
      total_fees,
      inscriptions,
    ))
  }

  pub(crate) fn create_batch_inscription_transactions(
    parent_info: Option<ParentInfo>,
    inscriptions: &Vec<Inscription>,
    wallet_inscriptions: BTreeMap<SatPoint, InscriptionId>,
    chain: Chain,
    mut utxos: BTreeMap<OutPoint, Amount>,
    change: [Address; 2],
    destinations: Vec<Address>,
    fee_rate: FeeRate,
    total_postage: Amount,
    batch_mode: Mode,
  ) -> Result<(Transaction, Transaction, TweakedKeyPair, u64)> {
    match batch_mode {
      Mode::SeparateOutputs => assert_eq!(
        destinations.len(),
        inscriptions.len(),
        "invariant: destination addresses and number of inscriptions doesn't match"
      ),
      Mode::SharedOutput => assert_eq!(
        destinations.len(),
        1,
        "invariant: destination addresses and number of inscriptions doesn't match"
      ),
    }

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

    let commit_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), chain.network());

    let mut reveal_inputs = vec![OutPoint::null()];
    let mut reveal_outputs = destinations
      .iter()
      .map(|destination| TxOut {
        script_pubkey: destination.script_pubkey(),
        value: match batch_mode {
          Mode::SeparateOutputs => TransactionBuilder::TARGET_POSTAGE.to_sat(),
          Mode::SharedOutput => total_postage.to_sat(),
        },
      })
      .collect::<Vec<TxOut>>();

    if let Some(ParentInfo {
      location,
      destination,
      tx_out,
    }) = parent_info.clone()
    {
      reveal_inputs.insert(0, location.outpoint);
      reveal_outputs.insert(
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
      fee_rate,
      reveal_inputs.clone(),
      commit_input,
      reveal_outputs.clone(),
      &reveal_script,
    );

    let unsigned_commit_tx = TransactionBuilder::new(
      satpoint,
      wallet_inscriptions,
      utxos.clone(),
      commit_tx_address.clone(),
      change,
      fee_rate,
      Target::Value(reveal_fee + total_postage),
    )
    .build_transaction()?;

    let (vout, _commit_output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
      .expect("should find sat commit/inscription output");

    reveal_inputs[commit_input] = OutPoint {
      txid: unsigned_commit_tx.txid(),
      vout: vout.try_into().unwrap(),
    };

    let (mut reveal_tx, _fee) = Inscribe::build_reveal_transaction(
      &control_block,
      fee_rate,
      reveal_inputs,
      commit_input,
      reveal_outputs.clone(),
      &reveal_script,
    );

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
        chain.network(),
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

  fn output(
    commit: Txid,
    reveal: Txid,
    batch_config: BatchConfig,
    total_fees: u64,
    inscriptions: Vec<Inscription>,
  ) -> crate::subcommand::wallet::inscribe::batch_inscribe::Output {
    let mut inscriptions_output = Vec::new();
    for index in 0..inscriptions.len() {
      let txid = reveal;
      let index = index.try_into().unwrap();
      let vout = if batch_config.parent.is_some() {
        index + 1
      } else {
        index
      };

      inscriptions_output.push(
        crate::subcommand::wallet::inscribe::batch_inscribe::InscriptionInfo {
          id: InscriptionId { txid, index },
          location: SatPoint {
            outpoint: OutPoint { txid, vout },
            offset: u64::from(index) * TransactionBuilder::TARGET_POSTAGE.to_sat(),
          },
        },
      )
    }

    crate::subcommand::wallet::inscribe::batch_inscribe::Output {
      commit,
      reveal,
      total_fees,
      parent: batch_config.parent,
      inscriptions: inscriptions_output,
    }
  }
}
