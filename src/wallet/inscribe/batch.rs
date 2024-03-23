use super::*;

pub struct Batch {
  pub(crate) commit_fee_rate: FeeRate,
  pub(crate) destinations: Vec<Address>,
  pub(crate) dry_run: bool,
  pub(crate) etch: Option<Etch>,
  pub(crate) inscriptions: Vec<Inscription>,
  pub(crate) mode: Mode,
  pub(crate) no_backup: bool,
  pub(crate) no_limit: bool,
  pub(crate) parent_info: Option<ParentInfo>,
  pub(crate) postages: Vec<Amount>,
  pub(crate) reinscribe: bool,
  pub(crate) reveal_fee_rate: FeeRate,
  pub(crate) reveal_satpoints: Vec<(SatPoint, TxOut)>,
  pub(crate) satpoint: Option<SatPoint>,
}

impl Default for Batch {
  fn default() -> Self {
    Self {
      commit_fee_rate: 1.0.try_into().unwrap(),
      destinations: Vec::new(),
      dry_run: false,
      etch: None,
      inscriptions: Vec::new(),
      mode: Mode::SharedOutput,
      no_backup: false,
      no_limit: false,
      parent_info: None,
      postages: vec![Amount::from_sat(10_000)],
      reinscribe: false,
      reveal_fee_rate: 1.0.try_into().unwrap(),
      reveal_satpoints: Vec::new(),
      satpoint: None,
    }
  }
}

impl Batch {
  pub(crate) fn inscribe(
    &self,
    locked_utxos: &BTreeSet<OutPoint>,
    runic_utxos: BTreeSet<OutPoint>,
    utxos: &BTreeMap<OutPoint, TxOut>,
    wallet: &Wallet,
  ) -> SubcommandResult {
    let BatchTransactions {
      commit_tx,
      reveal_tx,
      recovery_key_pair,
      total_fees,
      rune,
    } = self.create_batch_transactions(
      wallet.inscriptions().clone(),
      wallet.chain(),
      locked_utxos.clone(),
      runic_utxos,
      utxos.clone(),
      [wallet.get_change_address()?, wallet.get_change_address()?],
      wallet.get_change_address()?,
    )?;

    if self.dry_run {
      let commit_psbt = wallet
        .bitcoin_client()
        .wallet_process_psbt(
          &base64::engine::general_purpose::STANDARD
            .encode(Psbt::from_unsigned_tx(Self::remove_witnesses(commit_tx.clone()))?.serialize()),
          Some(false),
          None,
          None,
        )?
        .psbt;

      let reveal_psbt = Psbt::from_unsigned_tx(Self::remove_witnesses(reveal_tx.clone()))?;

      return Ok(Some(Box::new(self.output(
        commit_tx.txid(),
        Some(commit_psbt),
        reveal_tx.txid(),
        Some(base64::engine::general_purpose::STANDARD.encode(reveal_psbt.serialize())),
        total_fees,
        self.inscriptions.clone(),
        rune,
      ))));
    }

    let signed_commit_tx = wallet
      .bitcoin_client()
      .sign_raw_transaction_with_wallet(&commit_tx, None, None)?
      .hex;

    let result = wallet.bitcoin_client().sign_raw_transaction_with_wallet(
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
    )?;

    ensure!(
      result.complete,
      format!("Failed to sign reveal transaction: {:?}", result.errors)
    );

    let signed_reveal_tx = result.hex;

    if !self.no_backup {
      Self::backup_recovery_key(wallet, recovery_key_pair)?;
    }

    let commit = wallet
      .bitcoin_client()
      .send_raw_transaction(&signed_commit_tx)?;

    if self.etch.is_some() {
      eprintln!("Waiting for rune commitment to mature…");

      loop {
        let transaction = wallet
          .bitcoin_client()
          .get_transaction(&commit_tx.txid(), Some(true))
          .into_option()?;

        if let Some(transaction) = transaction {
          if u32::try_from(transaction.info.confirmations).unwrap() < RUNE_COMMIT_INTERVAL {
            continue;
          }
        }

        let tx_out = wallet
          .bitcoin_client()
          .get_tx_out(&commit_tx.txid(), 0, Some(true))?;

        if let Some(tx_out) = tx_out {
          if tx_out.confirmations >= RUNE_COMMIT_INTERVAL {
            break;
          }
        }

        if !wallet.integration_test() {
          thread::sleep(Duration::from_secs(5));
        }
      }
    }

    let reveal = match wallet
      .bitcoin_client()
      .send_raw_transaction(&signed_reveal_tx)
    {
      Ok(txid) => txid,
      Err(err) => {
        return Err(anyhow!(
        "Failed to send reveal transaction: {err}\nCommit tx {commit} will be recovered once mined"
      ))
      }
    };

    Ok(Some(Box::new(self.output(
      commit,
      None,
      reveal,
      None,
      total_fees,
      self.inscriptions.clone(),
      rune,
    ))))
  }

  fn remove_witnesses(mut transaction: Transaction) -> Transaction {
    for txin in transaction.input.iter_mut() {
      txin.witness = Witness::new();
    }

    transaction
  }

  fn output(
    &self,
    commit: Txid,
    commit_psbt: Option<String>,
    reveal: Txid,
    reveal_psbt: Option<String>,
    total_fees: u64,
    inscriptions: Vec<Inscription>,
    rune: Option<RuneInfo>,
  ) -> Output {
    let mut inscriptions_output = Vec::new();
    for i in 0..inscriptions.len() {
      let index = u32::try_from(i).unwrap();

      let vout = match self.mode {
        Mode::SharedOutput | Mode::SameSat => {
          if self.parent_info.is_some() {
            1
          } else {
            0
          }
        }
        Mode::SeparateOutputs | Mode::SatPoints => {
          if self.parent_info.is_some() {
            index + 1
          } else {
            index
          }
        }
      };

      let offset = match self.mode {
        Mode::SharedOutput => self.postages[0..i]
          .iter()
          .map(|amount| amount.to_sat())
          .sum(),
        Mode::SeparateOutputs | Mode::SameSat | Mode::SatPoints => 0,
      };

      let destination = match self.mode {
        Mode::SameSat | Mode::SharedOutput => &self.destinations[0],
        Mode::SatPoints | Mode::SeparateOutputs => &self.destinations[i],
      };

      inscriptions_output.push(InscriptionInfo {
        id: InscriptionId {
          txid: reveal,
          index,
        },
        destination: uncheck(destination),
        location: SatPoint {
          outpoint: OutPoint { txid: reveal, vout },
          offset,
        },
      });
    }

    Output {
      commit,
      commit_psbt,
      inscriptions: inscriptions_output,
      parent: self.parent_info.clone().map(|info| info.id),
      reveal,
      reveal_psbt,
      rune,
      total_fees,
    }
  }

  pub(crate) fn create_batch_transactions(
    &self,
    wallet_inscriptions: BTreeMap<SatPoint, Vec<InscriptionId>>,
    chain: Chain,
    locked_utxos: BTreeSet<OutPoint>,
    runic_utxos: BTreeSet<OutPoint>,
    mut utxos: BTreeMap<OutPoint, TxOut>,
    commit_change: [Address; 2],
    reveal_change: Address,
  ) -> Result<BatchTransactions> {
    if let Some(parent_info) = &self.parent_info {
      for inscription in &self.inscriptions {
        assert_eq!(inscription.parents(), vec![parent_info.id]);
      }
    }

    match self.mode {
      Mode::SameSat => {
        assert_eq!(
          self.postages.len(),
          1,
          "invariant: same-sat has only one postage"
        );
        assert_eq!(
          self.destinations.len(),
          1,
          "invariant: same-sat has only one destination"
        );
      }
      Mode::SeparateOutputs | Mode::SatPoints => {
        assert_eq!(
          self.destinations.len(),
          self.inscriptions.len(),
          "invariant: destination addresses and number of inscriptions doesn't match"
        );
        assert_eq!(
          self.destinations.len(),
          self.postages.len(),
          "invariant: destination addresses and number of postages doesn't match"
        );
      }
      Mode::SharedOutput => {
        assert_eq!(
          self.destinations.len(),
          1,
          "invariant: shared-output has only one destination"
        );
        assert_eq!(
          self.postages.len(),
          self.inscriptions.len(),
          "invariant: postages and number of inscriptions doesn't match"
        );
      }
    }

    let satpoint = if let Some(satpoint) = self.satpoint {
      satpoint
    } else {
      let inscribed_utxos = wallet_inscriptions
        .keys()
        .map(|satpoint| satpoint.outpoint)
        .collect::<BTreeSet<OutPoint>>();

      utxos
        .iter()
        .find(|(outpoint, txout)| {
          txout.value > 0
            && !inscribed_utxos.contains(outpoint)
            && !locked_utxos.contains(outpoint)
            && !runic_utxos.contains(outpoint)
        })
        .map(|(outpoint, _amount)| SatPoint {
          outpoint: *outpoint,
          offset: 0,
        })
        .ok_or_else(|| anyhow!("wallet contains no cardinal utxos"))?
    };

    let mut reinscription = false;

    for (inscribed_satpoint, inscription_ids) in &wallet_inscriptions {
      if *inscribed_satpoint == satpoint {
        reinscription = true;
        if self.reinscribe {
          continue;
        }

        bail!("sat at {} already inscribed", satpoint);
      }

      if inscribed_satpoint.outpoint == satpoint.outpoint {
        bail!(
          "utxo {} with sat {inscribed_satpoint} already inscribed with the following inscriptions:\n{}",
          satpoint.outpoint,
          inscription_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n"),
        );
      }
    }

    if self.reinscribe && !reinscription {
      bail!("reinscribe flag set but this would not be a reinscription");
    }

    let secp256k1 = Secp256k1::new();
    let key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let reveal_script = Inscription::append_batch_reveal_script(
      &self.inscriptions,
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

    let total_postage = self.postages.iter().map(|amount| amount.to_sat()).sum();

    let mut reveal_inputs = Vec::new();
    let mut reveal_outputs = Vec::new();

    if let Some(ParentInfo {
      location,
      id: _,
      destination,
      tx_out,
    }) = self.parent_info.clone()
    {
      reveal_inputs.push(location.outpoint);
      reveal_outputs.push(TxOut {
        script_pubkey: destination.script_pubkey(),
        value: tx_out.value,
      });
    }

    if self.mode == Mode::SatPoints {
      for (satpoint, _txout) in self.reveal_satpoints.iter() {
        reveal_inputs.push(satpoint.outpoint);
      }
    }

    reveal_inputs.push(OutPoint::null());

    for (i, destination) in self.destinations.iter().enumerate() {
      reveal_outputs.push(TxOut {
        script_pubkey: destination.script_pubkey(),
        value: match self.mode {
          Mode::SeparateOutputs | Mode::SatPoints => self.postages[i].to_sat(),
          Mode::SharedOutput | Mode::SameSat => total_postage,
        },
      });
    }

    let rune;
    let premine;
    let runestone;

    if let Some(etch) = self.etch {
      let mut edicts = Vec::new();

      let vout;
      let destination;
      premine = etch.premine.to_amount(etch.divisibility)?;

      if premine > 0 {
        let output = u32::try_from(reveal_outputs.len()).unwrap();
        destination = Some(reveal_change.clone());

        reveal_outputs.push(TxOut {
          script_pubkey: reveal_change.into(),
          value: TARGET_POSTAGE.to_sat(),
        });

        edicts.push(Edict {
          id: RuneId::default(),
          amount: premine,
          output,
        });

        vout = Some(output);
      } else {
        vout = None;
        destination = None;
      }

      let inner = Runestone {
        cenotaph: false,
        claim: None,
        default_output: None,
        edicts,
        etching: Some(Etching {
          divisibility: etch.divisibility,
          mint: etch
            .mint
            .map(|mint| -> Result<runes::Mint> {
              Ok(runes::Mint {
                deadline: mint.deadline,
                term: mint.term,
                limit: Some(mint.limit.to_amount(etch.divisibility)?),
              })
            })
            .transpose()?,
          rune: Some(etch.rune.rune),
          spacers: etch.rune.spacers,
          symbol: Some(etch.symbol),
        }),
      };

      let script_pubkey = inner.encipher();

      runestone = Some(inner);

      ensure!(
        script_pubkey.len() <= 82,
        "runestone greater than maximum OP_RETURN size: {} > 82",
        script_pubkey.len()
      );

      reveal_outputs.push(TxOut {
        script_pubkey,
        value: 0,
      });

      rune = Some((destination, etch.rune, vout));
    } else {
      premine = 0;
      rune = None;
      runestone = None;
    }

    let commit_input = usize::from(self.parent_info.is_some()) + self.reveal_satpoints.len();

    let (_reveal_tx, reveal_fee) = Self::build_reveal_transaction(
      commit_input,
      &control_block,
      self.reveal_fee_rate,
      reveal_outputs.clone(),
      reveal_inputs.clone(),
      &reveal_script,
    );

    let mut target_value = reveal_fee;

    if self.mode != Mode::SatPoints {
      target_value += Amount::from_sat(total_postage);
    }

    if premine > 0 {
      target_value += TARGET_POSTAGE;
    }

    let unsigned_commit_tx = TransactionBuilder::new(
      satpoint,
      wallet_inscriptions,
      utxos.clone(),
      locked_utxos.clone(),
      runic_utxos,
      commit_tx_address.clone(),
      commit_change,
      self.commit_fee_rate,
      Target::Value(target_value),
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

    let (mut reveal_tx, _fee) = Self::build_reveal_transaction(
      commit_input,
      &control_block,
      self.reveal_fee_rate,
      reveal_outputs.clone(),
      reveal_inputs,
      &reveal_script,
    );

    for output in reveal_tx.output.iter() {
      ensure!(
        output.value >= output.script_pubkey.dust_value().to_sat(),
        "commit transaction output would be dust"
      );
    }

    let mut prevouts = Vec::new();

    if let Some(parent_info) = self.parent_info.clone() {
      prevouts.push(parent_info.tx_out);
    }

    if self.mode == Mode::SatPoints {
      for (_satpoint, txout) in self.reveal_satpoints.iter() {
        prevouts.push(txout.clone());
      }
    }

    prevouts.push(unsigned_commit_tx.output[vout].clone());

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

    if !self.no_limit && reveal_weight > bitcoin::Weight::from_wu(MAX_STANDARD_TX_WEIGHT.into()) {
      bail!(
        "reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {reveal_weight}"
      );
    }

    utxos.insert(
      reveal_tx.input[commit_input].previous_output,
      unsigned_commit_tx.output[reveal_tx.input[commit_input].previous_output.vout as usize]
        .clone(),
    );

    let total_fees =
      Self::calculate_fee(&unsigned_commit_tx, &utxos) + Self::calculate_fee(&reveal_tx, &utxos);

    match (Runestone::from_transaction(&reveal_tx), runestone) {
      (Some(actual), Some(expected)) => pretty_assert_eq!(
        actual,
        expected,
        "commit transaction runestone did not match expected runestone"
      ),
      (Some(_), None) => panic!("commit transaction contained runestone, but none was expected"),
      (None, Some(_)) => {
        panic!("commit transaction did not contain runestone, but one was expected")
      }
      (None, None) => {}
    }

    let rune = rune.map(|(destination, rune, vout)| RuneInfo {
      destination: destination.map(|destination| uncheck(&destination)),
      location: vout.map(|vout| OutPoint {
        txid: reveal_tx.txid(),
        vout,
      }),
      rune,
    });

    Ok(BatchTransactions {
      commit_tx: unsigned_commit_tx,
      recovery_key_pair,
      reveal_tx,
      rune,
      total_fees,
    })
  }

  fn backup_recovery_key(wallet: &Wallet, recovery_key_pair: TweakedKeyPair) -> Result {
    let recovery_private_key = PrivateKey::new(
      recovery_key_pair.to_inner().secret_key(),
      wallet.chain().network(),
    );

    let info = wallet
      .bitcoin_client()
      .get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_wif()))?;

    let response = wallet
      .bitcoin_client()
      .import_descriptors(vec![ImportDescriptors {
        descriptor: format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
        timestamp: Timestamp::Now,
        active: Some(false),
        range: None,
        next_index: None,
        internal: Some(false),
        label: Some("commit tx recovery key".to_string()),
      }])?;

    for result in response {
      if !result.success {
        return Err(anyhow!("commit tx recovery key import failed"));
      }
    }

    Ok(())
  }

  fn build_reveal_transaction(
    commit_input_index: usize,
    control_block: &ControlBlock,
    fee_rate: FeeRate,
    output: Vec<TxOut>,
    input: Vec<OutPoint>,
    script: &Script,
  ) -> (Transaction, Amount) {
    let reveal_tx = Transaction {
      input: input
        .into_iter()
        .map(|previous_output| TxIn {
          previous_output,
          script_sig: script::Builder::new().into_script(),
          witness: Witness::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        })
        .collect(),
      output,
      lock_time: LockTime::ZERO,
      version: 2,
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

  fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, TxOut>) -> u64 {
    tx.input
      .iter()
      .map(|txin| utxos.get(&txin.previous_output).unwrap().value)
      .sum::<u64>()
      .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<u64>())
      .unwrap()
  }
}
