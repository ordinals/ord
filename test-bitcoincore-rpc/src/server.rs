use {
  super::*,
  bitcoin::{
    secp256k1::{rand, KeyPair, Secp256k1, XOnlyPublicKey},
    Address, Witness,
  },
  serde_json::json,
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
      version: 230000,
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
          version: 0,
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
      wallet_name: self.state().wallet_name.clone(),
      wallet_version: 0,
    })
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
      lock_time: PackedLockTime(0),
      input: utxos
        .iter()
        .map(|input| TxIn {
          previous_output: OutPoint::new(input.txid, input.vout),
          script_sig: Script::new(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        })
        .collect(),
      output: outs
        .iter()
        .map(|(_address, amount)| TxOut {
          value: (*amount * COIN_VALUE as f64) as u64,
          script_pubkey: Script::new(),
        })
        .collect(),
    };

    Ok(hex::encode(serialize(&tx)))
  }

  fn sign_raw_transaction_with_wallet(
    &self,
    tx: String,
    utxos: Option<()>,
    sighash_type: Option<()>,
  ) -> Result<Value, jsonrpc_core::Error> {
    assert_eq!(utxos, None, "utxos param not supported");
    assert_eq!(sighash_type, None, "sighash_type param not supported");

    Ok(
      serde_json::to_value(SignRawTransactionResult {
        hex: hex::decode(tx).unwrap(),
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
    verbose: bool,
    blockhash: Option<BlockHash>,
  ) -> Result<Value, jsonrpc_core::Error> {
    assert_eq!(blockhash, None, "Blockhash param is unsupported");
    if verbose {
      match self.state().transactions.get(&txid) {
        Some(_) => Ok(
          serde_json::to_value(GetRawTransactionResult {
            in_active_chain: None,
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
    address: Option<bitcoin::Address>,
    include_unsafe: Option<bool>,
    query_options: Option<String>,
  ) -> Result<Vec<ListUnspentResultEntry>, jsonrpc_core::Error> {
    assert_eq!(minconf, None, "minconf param not supported");
    assert_eq!(maxconf, None, "maxconf param not supported");
    assert_eq!(address, None, "address param not supported");
    assert_eq!(include_unsafe, None, "include_unsafe param not supported");
    assert_eq!(query_options, None, "query_options param not supported");
    Ok(
      self
        .state()
        .utxos
        .iter()
        .map(|(outpoint, &amount)| ListUnspentResultEntry {
          txid: outpoint.txid,
          vout: outpoint.vout,
          address: None,
          label: None,
          redeem_script: None,
          witness_script: None,
          script_pub_key: Script::new(),
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

  fn get_raw_change_address(&self) -> Result<bitcoin::Address, jsonrpc_core::Error> {
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
    _params: Vec<serde_json::Value>,
  ) -> Result<serde_json::Value, jsonrpc_core::Error> {
    Ok(json!([{"success": true}]))
  }

  fn get_new_address(
    &self,
    _label: Option<String>,
    _address_type: Option<()>,
  ) -> Result<bitcoin::Address, jsonrpc_core::Error> {
    let secp256k1 = Secp256k1::new();
    let key_pair = KeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
    let address = Address::p2tr(&secp256k1, public_key, None, self.network);

    Ok(address)
  }
}
