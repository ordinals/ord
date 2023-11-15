use {
  super::*,
  bitcoin::{
    secp256k1::{rand, KeyPair, Secp256k1, XOnlyPublicKey},
    Witness,
  },
  bitcoincore_rpc::RawTx,
};

pub(crate) struct Server {
  pub(crate) state: Arc<Mutex<State>>,
  pub(crate) network: Network,
}

impl Server {
  pub(crate) fn new(state: Arc<Mutex<State>>) -> Self {
    let network = state.lock().unwrap().network;
    Self { network, state }
  }

  fn state(&self) -> MutexGuard<State> {
    self.state.lock().unwrap()
  }

  fn not_found() -> jsonrpc_core::Error {
    jsonrpc_core::Error::new(jsonrpc_core::types::error::ErrorCode::ServerError(-8))
  }
}

impl Api for Server {
  fn get_balances(&self) -> Result<GetBalancesResult, jsonrpc_core::Error> {
    Ok(GetBalancesResult {
      mine: GetBalancesResultEntry {
        immature: Amount::from_sat(0),
        trusted: self
          .list_unspent(None, None, None, None, None)?
          .iter()
          .map(|entry| entry.amount)
          .sum(),
        untrusted_pending: Amount::from_sat(0),
      },
      watchonly: None,
    })
  }

  fn get_blockchain_info(&self) -> Result<GetBlockchainInfoResult, jsonrpc_core::Error> {
    Ok(GetBlockchainInfoResult {
      chain: String::from(match self.network {
        Network::Bitcoin => "main",
        Network::Testnet => "test",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
        _ => panic!(),
      }),
      blocks: 0,
      headers: 0,
      best_block_hash: self.state().hashes[0],
      difficulty: 0.0,
      median_time: 0,
      verification_progress: 0.0,
      initial_block_download: false,
      chain_work: Vec::new(),
      size_on_disk: 0,
      pruned: false,
      prune_height: None,
      automatic_pruning: None,
      prune_target_size: None,
      softforks: HashMap::new(),
      warnings: String::new(),
    })
  }

  fn get_network_info(&self) -> Result<GetNetworkInfoResult, jsonrpc_core::Error> {
    Ok(GetNetworkInfoResult {
      version: self.state().version,
      subversion: String::new(),
      protocol_version: 0,
      local_services: String::new(),
      local_relay: false,
      time_offset: 0,
      connections: 0,
      connections_in: None,
      connections_out: None,
      network_active: true,
      networks: Vec::new(),
      relay_fee: Amount::from_sat(0),
      incremental_fee: Amount::from_sat(0),
      local_addresses: Vec::new(),
      warnings: String::new(),
    })
  }

  fn get_block_hash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.state().hashes.get(height) {
      Some(block_hash) => Ok(*block_hash),
      None => Err(Self::not_found()),
    }
  }

  fn get_block_header(
    &self,
    block_hash: BlockHash,
    verbose: bool,
  ) -> Result<Value, jsonrpc_core::Error> {
    if verbose {
      let height = match self
        .state()
        .hashes
        .iter()
        .position(|hash| *hash == block_hash)
      {
        Some(height) => height,
        None => return Err(Self::not_found()),
      };

      Ok(
        serde_json::to_value(GetBlockHeaderResult {
          bits: String::new(),
          chainwork: Vec::new(),
          confirmations: 0,
          difficulty: 0.0,
          hash: block_hash,
          height,
          median_time: None,
          merkle_root: TxMerkleNode::all_zeros(),
          n_tx: 0,
          next_block_hash: None,
          nonce: 0,
          previous_block_hash: None,
          time: 0,
          version: Version::ONE,
          version_hex: Some(vec![0, 0, 0, 0]),
        })
        .unwrap(),
      )
    } else {
      match self.state().blocks.get(&block_hash) {
        Some(block) => Ok(serde_json::to_value(hex::encode(serialize(&block.header))).unwrap()),
        None => Err(Self::not_found()),
      }
    }
  }

  fn get_block(
    &self,
    block_hash: BlockHash,
    verbosity: u64,
  ) -> Result<String, jsonrpc_core::Error> {
    assert_eq!(verbosity, 0, "Verbosity level {verbosity} is unsupported");
    match self.state().blocks.get(&block_hash) {
      Some(block) => Ok(hex::encode(serialize(block))),
      None => Err(Self::not_found()),
    }
  }

  fn get_block_count(&self) -> Result<u64, jsonrpc_core::Error> {
    Ok(
      self
        .state()
        .hashes
        .len()
        .saturating_sub(1)
        .try_into()
        .unwrap(),
    )
  }

  fn get_wallet_info(&self) -> Result<GetWalletInfoResult, jsonrpc_core::Error> {
    if let Some(wallet_name) = self.state().loaded_wallets.first().cloned() {
      Ok(GetWalletInfoResult {
        avoid_reuse: None,
        balance: Amount::from_sat(0),
        hd_seed_id: None,
        immature_balance: Amount::from_sat(0),
        keypool_oldest: None,
        keypool_size: 0,
        keypool_size_hd_internal: 0,
        pay_tx_fee: Amount::from_sat(0),
        private_keys_enabled: false,
        scanning: None,
        tx_count: 0,
        unconfirmed_balance: Amount::from_sat(0),
        unlocked_until: None,
        wallet_name,
        wallet_version: 0,
      })
    } else {
      Err(Self::not_found())
    }
  }

  fn create_raw_transaction(
    &self,
    utxos: Vec<CreateRawTransactionInput>,
    outs: HashMap<String, f64>,
    locktime: Option<i64>,
    replaceable: Option<bool>,
  ) -> Result<String, jsonrpc_core::Error> {
    assert_eq!(locktime, None, "locktime param not supported");
    assert_eq!(replaceable, None, "replaceable param not supported");

    let tx = Transaction {
      version: 0,
      lock_time: LockTime::ZERO,
      input: utxos
        .iter()
        .map(|input| TxIn {
          previous_output: OutPoint::new(input.txid, input.vout),
          script_sig: ScriptBuf::new(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        })
        .collect(),
      output: outs
        .values()
        .map(|amount| TxOut {
          value: (*amount * COIN_VALUE as f64) as u64,
          script_pubkey: ScriptBuf::new(),
        })
        .collect(),
    };

    Ok(hex::encode(serialize(&tx)))
  }

  fn create_wallet(
    &self,
    name: String,
    _disable_private_keys: Option<bool>,
    _blank: Option<bool>,
    _passphrase: Option<String>,
    _avoid_reuse: Option<bool>,
  ) -> Result<LoadWalletResult, jsonrpc_core::Error> {
    self.state().wallets.insert(name.clone());
    Ok(LoadWalletResult {
      name,
      warning: None,
    })
  }

  fn sign_raw_transaction_with_wallet(
    &self,
    tx: String,
    _utxos: Option<Vec<SignRawTransactionInput>>,
    sighash_type: Option<()>,
  ) -> Result<Value, jsonrpc_core::Error> {
    assert_eq!(sighash_type, None, "sighash_type param not supported");

    let mut transaction: Transaction = deserialize(&hex::decode(tx).unwrap()).unwrap();
    for input in &mut transaction.input {
      if input.witness.is_empty() {
        input.witness = Witness::from_slice(&[&[0; 64]]);
      }
    }

    Ok(
      serde_json::to_value(SignRawTransactionResult {
        hex: hex::decode(transaction.raw_hex()).unwrap(),
        complete: true,
        errors: None,
      })
      .unwrap(),
    )
  }

  fn send_raw_transaction(&self, tx: String) -> Result<String, jsonrpc_core::Error> {
    let tx: Transaction = deserialize(&hex::decode(tx).unwrap()).unwrap();
    self.state.lock().unwrap().mempool.push(tx.clone());

    Ok(tx.txid().to_string())
  }

  fn send_to_address(
    &self,
    address: Address<NetworkUnchecked>,
    amount: f64,
    comment: Option<String>,
    comment_to: Option<String>,
    subtract_fee: Option<bool>,
    replaceable: Option<bool>,
    confirmation_target: Option<u32>,
    estimate_mode: Option<EstimateMode>,
    avoid_reuse: Option<bool>,
    fee_rate: Option<f64>,
    verbose: Option<bool>,
  ) -> Result<Txid, jsonrpc_core::Error> {
    assert_eq!(comment, None);
    assert_eq!(comment_to, None);
    assert_eq!(subtract_fee, None);
    assert_eq!(replaceable, None);
    assert_eq!(confirmation_target, None);
    assert_eq!(estimate_mode, None);
    assert_eq!(avoid_reuse, None);
    assert_eq!(verbose, None);

    let mut state = self.state.lock().unwrap();
    let locked = state.locked.iter().cloned().collect::<Vec<OutPoint>>();

    let value = Amount::from_btc(amount).expect("error converting amount to sat");

    let (outpoint, utxo_amount) = match state
      .utxos
      .iter()
      .find(|(outpoint, amount)| *amount >= &value && !locked.contains(outpoint))
    {
      Some((outpoint, utxo_amount)) => (outpoint, utxo_amount),
      _ => return Err(Self::not_found()),
    };

    let mut transaction = Transaction {
      version: 1,
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: *outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: vec![
        TxOut {
          value: value.to_sat(),
          script_pubkey: address.payload.script_pubkey(),
        },
        TxOut {
          value: (*utxo_amount - value).to_sat(),
          script_pubkey: address.payload.script_pubkey(),
        },
      ],
    };

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let fee = (fee_rate.unwrap_or(1.0) * transaction.vsize() as f64).round() as u64;

    transaction.output[1].value -= fee;

    state.mempool.push(transaction);

    state.sent.push(Sent {
      address: address.assume_checked(),
      amount,
      locked,
    });

    Ok(
      "0000000000000000000000000000000000000000000000000000000000000000"
        .parse()
        .unwrap(),
    )
  }

  fn get_transaction(
    &self,
    txid: Txid,
    _include_watchonly: Option<bool>,
  ) -> Result<Value, jsonrpc_core::Error> {
    match self.state.lock().unwrap().transactions.get(&txid) {
      Some(tx) => Ok(
        serde_json::to_value(GetTransactionResult {
          info: WalletTxInfo {
            txid,
            confirmations: 0,
            time: 0,
            timereceived: 0,
            blockhash: None,
            blockindex: None,
            blockheight: None,
            blocktime: None,
            wallet_conflicts: Vec::new(),
            bip125_replaceable: Bip125Replaceable::Unknown,
          },
          amount: SignedAmount::from_sat(0),
          fee: None,
          details: Vec::new(),
          hex: serialize(tx),
        })
        .unwrap(),
      ),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn get_raw_transaction(
    &self,
    txid: Txid,
    verbose: Option<bool>,
    blockhash: Option<BlockHash>,
  ) -> Result<Value, jsonrpc_core::Error> {
    assert_eq!(blockhash, None, "Blockhash param is unsupported");
    if verbose.unwrap_or(false) {
      match self.state().transactions.get(&txid) {
        Some(_) => Ok(
          serde_json::to_value(GetRawTransactionResult {
            in_active_chain: Some(true),
            hex: Vec::new(),
            txid: Txid::all_zeros(),
            hash: Wtxid::all_zeros(),
            size: 0,
            vsize: 0,
            version: 0,
            locktime: 0,
            vin: Vec::new(),
            vout: Vec::new(),
            blockhash: None,
            confirmations: Some(1),
            time: None,
            blocktime: None,
          })
          .unwrap(),
        ),
        None => Err(Self::not_found()),
      }
    } else {
      match self.state().transactions.get(&txid) {
        Some(tx) => Ok(Value::String(hex::encode(serialize(tx)))),
        None => Err(Self::not_found()),
      }
    }
  }

  fn list_unspent(
    &self,
    minconf: Option<usize>,
    maxconf: Option<usize>,
    address: Option<Address<NetworkUnchecked>>,
    include_unsafe: Option<bool>,
    query_options: Option<String>,
  ) -> Result<Vec<ListUnspentResultEntry>, jsonrpc_core::Error> {
    assert_eq!(minconf, None, "minconf param not supported");
    assert_eq!(maxconf, None, "maxconf param not supported");
    assert_eq!(address, None, "address param not supported");
    assert_eq!(include_unsafe, None, "include_unsafe param not supported");
    assert_eq!(query_options, None, "query_options param not supported");

    let state = self.state();

    Ok(
      state
        .utxos
        .iter()
        .filter(|(outpoint, _amount)| !state.locked.contains(outpoint))
        .map(|(outpoint, &amount)| ListUnspentResultEntry {
          txid: outpoint.txid,
          vout: outpoint.vout,
          address: None,
          label: None,
          redeem_script: None,
          witness_script: None,
          script_pub_key: ScriptBuf::new(),
          amount,
          confirmations: 0,
          spendable: true,
          solvable: true,
          descriptor: None,
          safe: true,
        })
        .collect(),
    )
  }

  fn list_lock_unspent(&self) -> Result<Vec<JsonOutPoint>, jsonrpc_core::Error> {
    Ok(
      self
        .state()
        .locked
        .iter()
        .map(|outpoint| (*outpoint).into())
        .collect(),
    )
  }

  fn get_raw_change_address(
    &self,
    _address_type: Option<bitcoincore_rpc::json::AddressType>,
  ) -> Result<Address, jsonrpc_core::Error> {
    let secp256k1 = Secp256k1::new();
    let key_pair = KeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
    let address = Address::p2tr(&secp256k1, public_key, None, self.network);

    Ok(address)
  }

  fn get_descriptor_info(
    &self,
    desc: String,
  ) -> Result<GetDescriptorInfoResult, jsonrpc_core::Error> {
    Ok(GetDescriptorInfoResult {
      descriptor: desc,
      checksum: "".into(),
      is_range: false,
      is_solvable: false,
      has_private_keys: true,
    })
  }

  fn import_descriptors(
    &self,
    req: Vec<ImportDescriptors>,
  ) -> Result<Vec<ImportMultiResult>, jsonrpc_core::Error> {
    self
      .state()
      .descriptors
      .extend(req.into_iter().map(|params| params.descriptor));

    Ok(vec![ImportMultiResult {
      success: true,
      warnings: Vec::new(),
      error: None,
    }])
  }

  fn get_new_address(
    &self,
    _label: Option<String>,
    _address_type: Option<bitcoincore_rpc::json::AddressType>,
  ) -> Result<bitcoin::Address, jsonrpc_core::Error> {
    let secp256k1 = Secp256k1::new();
    let key_pair = KeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
    let address = Address::p2tr(&secp256k1, public_key, None, self.network);

    Ok(address)
  }

  fn list_transactions(
    &self,
    _label: Option<String>,
    count: Option<u16>,
    _skip: Option<usize>,
    _include_watchonly: Option<bool>,
  ) -> Result<Vec<ListTransactionResult>, jsonrpc_core::Error> {
    let state = self.state();
    Ok(
      state
        .transactions
        .iter()
        .take(count.unwrap_or(u16::MAX).into())
        .map(|(txid, tx)| (*txid, tx))
        .chain(state.mempool.iter().map(|tx| (tx.txid(), tx)))
        .map(|(txid, tx)| ListTransactionResult {
          info: WalletTxInfo {
            confirmations: state.get_confirmations(tx),
            blockhash: None,
            blockindex: None,
            blocktime: None,
            blockheight: None,
            txid,
            time: 0,
            timereceived: 0,
            bip125_replaceable: Bip125Replaceable::Unknown,
            wallet_conflicts: Vec::new(),
          },
          detail: GetTransactionResultDetail {
            address: None,
            category: GetTransactionResultDetailCategory::Immature,
            amount: SignedAmount::from_sat(0),
            label: None,
            vout: 0,
            fee: Some(SignedAmount::from_sat(0)),
            abandoned: None,
          },
          trusted: None,
          comment: None,
        })
        .collect(),
    )
  }

  fn lock_unspent(
    &self,
    unlock: bool,
    outputs: Vec<JsonOutPoint>,
  ) -> Result<bool, jsonrpc_core::Error> {
    assert!(!unlock);

    let mut state = self.state();

    if state.fail_lock_unspent {
      return Ok(false);
    }

    for output in outputs {
      let output = OutPoint {
        vout: output.vout,
        txid: output.txid,
      };
      assert!(state.utxos.contains_key(&output));
      state.locked.insert(output);
    }

    Ok(true)
  }

  fn list_descriptors(&self) -> Result<ListDescriptorsResult, jsonrpc_core::Error> {
    Ok(ListDescriptorsResult {
      wallet_name: "ord".into(),
      descriptors: self
        .state()
        .descriptors
        .iter()
        .map(|desc| Descriptor {
          desc: desc.to_string(),
          timestamp: Timestamp::Now,
          active: true,
          internal: None,
          range: None,
          next: None,
        })
        .collect(),
    })
  }

  fn load_wallet(&self, wallet: String) -> Result<LoadWalletResult, jsonrpc_core::Error> {
    if self.state().wallets.contains(&wallet) {
      self.state().loaded_wallets.insert(wallet.clone());
      Ok(LoadWalletResult {
        name: wallet,
        warning: None,
      })
    } else {
      Err(Self::not_found())
    }
  }

  fn list_wallets(&self) -> Result<Vec<String>, jsonrpc_core::Error> {
    Ok(
      self
        .state()
        .loaded_wallets
        .clone()
        .into_iter()
        .collect::<Vec<String>>(),
    )
  }
}
