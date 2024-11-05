use {
  super::*,
  base64::Engine,
  bitcoin::{consensus::Decodable, psbt::Psbt, Witness},
  bitcoincore_rpc::json::StringOrStringArray,
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

  fn get_best_block_hash(&self) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.state().hashes.last() {
      Some(block_hash) => Ok(*block_hash),
      None => Err(Self::not_found()),
    }
  }

  fn get_blockchain_info(&self) -> Result<GetBlockchainInfoResult, jsonrpc_core::Error> {
    Ok(GetBlockchainInfoResult {
      chain: self.network,
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
      warnings: StringOrStringArray::String(String::new()),
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
      warnings: StringOrStringArray::String(String::new()),
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
          chainwork: hex::decode(
            "0000000000000000000000000000000000000000000000000000000000000000",
          )
          .unwrap(),
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
          version: bitcoin::blockdata::block::Version::ONE,
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

  fn get_block_header_info(
    &self,
    block_hash: BlockHash,
  ) -> Result<GetBlockHeaderResult, jsonrpc_core::Error> {
    let state = self.state();

    let height = match state.hashes.iter().position(|hash| *hash == block_hash) {
      Some(height) => height,
      None => return Err(Self::not_found()),
    };

    Ok(GetBlockHeaderResult {
      height,
      hash: block_hash,
      confirmations: 0,
      version: bitcoin::block::Version::ONE,
      version_hex: None,
      merkle_root: TxMerkleNode::all_zeros(),
      time: 0,
      median_time: None,
      nonce: 0,
      bits: String::new(),
      difficulty: 0.0,
      chainwork: Vec::new(),
      n_tx: 0,
      previous_block_hash: None,
      next_block_hash: None,
    })
  }

  fn get_block_stats(&self, height: usize) -> Result<GetBlockStatsResult, jsonrpc_core::Error> {
    let Some(block_hash) = self.state().hashes.get(height).cloned() else {
      return Err(Self::not_found());
    };

    Ok(GetBlockStatsResult {
      avg_fee: Amount::ZERO,
      avg_fee_rate: Amount::ZERO,
      avg_tx_size: 0,
      block_hash,
      fee_rate_percentiles: FeeRatePercentiles {
        fr_10th: Amount::ZERO,
        fr_25th: Amount::ZERO,
        fr_50th: Amount::ZERO,
        fr_75th: Amount::ZERO,
        fr_90th: Amount::ZERO,
      },
      height: height.try_into().unwrap(),
      ins: 0,
      max_fee: Amount::ZERO,
      max_fee_rate: Amount::ZERO,
      max_tx_size: 0,
      median_fee: Amount::ZERO,
      median_time: 0,
      median_tx_size: 0,
      min_fee: Amount::ZERO,
      min_fee_rate: Amount::ZERO,
      min_tx_size: 0,
      outs: 0,
      subsidy: Amount::ZERO,
      sw_total_size: 0,
      sw_total_weight: 0,
      sw_txs: 0,
      time: 0,
      total_out: Amount::ZERO,
      total_size: 0,
      total_weight: 0,
      total_fee: Amount::ZERO,
      txs: 0,
      utxo_increase: 0,
      utxo_size_inc: 0,
    })
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

  fn get_tx_out(
    &self,
    txid: Txid,
    vout: u32,
    _include_mempool: Option<bool>,
  ) -> Result<Option<GetTxOutResult>, jsonrpc_core::Error> {
    let state = self.state();

    let Some(value) = state.utxos.get(&OutPoint { txid, vout }) else {
      return Ok(None);
    };

    let mut confirmations = None;

    for (height, hash) in state.hashes.iter().enumerate() {
      for tx in &state.blocks[hash].txdata {
        if tx.compute_txid() == txid {
          confirmations = Some(state.hashes.len() - height);
        }
      }
    }

    Ok(Some(GetTxOutResult {
      bestblock: BlockHash::all_zeros(),
      coinbase: false,
      confirmations: confirmations.unwrap().try_into().unwrap(),
      script_pub_key: GetRawTransactionResultVoutScriptPubKey {
        asm: String::new(),
        hex: Vec::new(),
        req_sigs: None,
        type_: None,
        addresses: Vec::new(),
        address: None,
      },
      value: *value,
    }))
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
        private_keys_enabled: true,
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
      version: Version(2),
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
          value: Amount::from_sat((*amount * COIN_VALUE as f64) as u64),
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

  fn fund_raw_transaction(
    &self,
    tx: String,
    options: Option<FundRawTransactionOptions>,
    _is_witness: Option<bool>,
  ) -> Result<FundRawTransactionResult, jsonrpc_core::Error> {
    let options = options.unwrap();

    let mut cursor = bitcoin::io::Cursor::new(hex::decode(tx).unwrap());

    let version = Version(i32::consensus_decode_from_finite_reader(&mut cursor).unwrap());
    let input = Vec::<TxIn>::consensus_decode_from_finite_reader(&mut cursor).unwrap();
    let output = Decodable::consensus_decode_from_finite_reader(&mut cursor).unwrap();
    let lock_time = Decodable::consensus_decode_from_finite_reader(&mut cursor).unwrap();

    let mut transaction = Transaction {
      version,
      input,
      output,
      lock_time,
    };

    assert_eq!(
      options.change_position,
      Some(transaction.output.len().try_into().unwrap())
    );

    let mut state = self.state();

    let output_value = transaction
      .output
      .iter()
      .map(|txout| txout.value.to_sat())
      .sum();

    let mut utxos = state
      .utxos
      .clone()
      .into_iter()
      .map(|(outpoint, value)| (value, outpoint))
      .collect::<Vec<(Amount, OutPoint)>>();

    let mut input_value = transaction
      .input
      .iter()
      .map(|txin| state.utxos.get(&txin.previous_output).unwrap().to_sat())
      .sum::<u64>();

    utxos.sort();
    utxos.reverse();

    if output_value > input_value {
      for (value, outpoint) in utxos {
        if state.locked.contains(&outpoint) {
          continue;
        }

        let tx = state.transactions.get(&outpoint.txid).unwrap();

        let tx_out = &tx.output[usize::try_from(outpoint.vout).unwrap()];

        let Ok(address) = Address::from_script(&tx_out.script_pubkey, state.network) else {
          continue;
        };

        if !state.is_wallet_address(&address) {
          continue;
        }

        transaction.input.push(TxIn {
          previous_output: outpoint,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::default(),
        });

        input_value += value.to_sat();

        if input_value > output_value {
          break;
        }
      }

      if output_value > input_value {
        return Err(jsonrpc_core::Error {
          code: jsonrpc_core::ErrorCode::ServerError(-6),
          message: "insufficient funds".into(),
          data: None,
        });
      }
    }

    let change_position = transaction.output.len() as i32;

    let change = input_value - output_value;

    if change > 0 {
      transaction.output.push(TxOut {
        value: Amount::from_sat(change),
        script_pubkey: state.new_address(true).into(),
      });
    }

    let fee = if let Some(fee_rate) = options.fee_rate {
      // increase vsize to account for the witness that `fundrawtransaction` will add
      let funded_vsize = transaction.vsize() as f64 + 68.0 / 4.0;
      let funded_kwu = funded_vsize / 1000.0;
      let fee = (funded_kwu * fee_rate.to_sat() as f64) as u64;
      transaction.output.last_mut().unwrap().value -= Amount::from_sat(fee);
      fee
    } else {
      0
    };

    Ok(FundRawTransactionResult {
      hex: serialize(&transaction),
      fee: Amount::from_sat(fee),
      change_position,
    })
  }

  fn sign_raw_transaction_with_wallet(
    &self,
    tx: String,
    utxos: Option<Vec<SignRawTransactionInput>>,
    sighash_type: Option<()>,
  ) -> Result<Value, jsonrpc_core::Error> {
    assert_eq!(sighash_type, None, "sighash_type param not supported");

    let mut transaction: Transaction = deserialize(&hex::decode(tx).unwrap()).unwrap();

    if let Some(utxos) = &utxos {
      // sign for zero-value UTXOs produced by `ord wallet sign`
      if utxos[0].amount == Some(Amount::ZERO) {
        transaction.input[0].witness = self.state().wallet.sign_bip322(&utxos[0], &transaction);
      }
    }

    for input in &mut transaction.input {
      if input.witness.is_empty() {
        input.witness = Witness::from_slice(&[&[0; 64]]);
      }
    }

    Ok(
      serde_json::to_value(SignRawTransactionResult {
        hex: serialize(&transaction),
        complete: true,
        errors: None,
      })
      .unwrap(),
    )
  }

  fn send_raw_transaction(&self, tx: String) -> Result<String, jsonrpc_core::Error> {
    let tx: Transaction = deserialize(&hex::decode(tx).unwrap()).unwrap();

    let mut state = self.state.lock().unwrap();

    for tx_in in &tx.input {
      if let Some(lock_time) = tx_in.sequence.to_relative_lock_time() {
        match lock_time {
          bitcoin::relative::LockTime::Blocks(blocks) => {
            if state
              .txid_to_block_height
              .get(&tx_in.previous_output.txid)
              .expect("input has not been miined")
              + u32::from(blocks.value())
              > u32::try_from(state.hashes.len()).unwrap()
            {
              panic!("input is locked");
            }
          }
          bitcoin::relative::LockTime::Time(_) => {
            panic!("time-based relative locktimes are not implemented")
          }
        }
      }
    }

    state.mempool.push(tx.clone());

    Ok(tx.compute_txid().to_string())
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
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: *outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: vec![
        TxOut {
          value,
          script_pubkey: address.assume_checked_ref().script_pubkey(),
        },
        TxOut {
          value: *utxo_amount - value,
          script_pubkey: address.assume_checked_ref().script_pubkey(),
        },
      ],
    };

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let fee = (fee_rate.unwrap_or(1.0) * transaction.vsize() as f64).round() as u64;

    transaction.output[1].value -= Amount::from_sat(fee);

    let txid = transaction.compute_txid();

    state.mempool.push(transaction);

    Ok(txid)
  }

  fn get_transaction(
    &self,
    txid: Txid,
    _include_watchonly: Option<bool>,
  ) -> Result<Value, jsonrpc_core::Error> {
    let state = self.state();

    let Some(tx) = state.transactions.get(&txid) else {
      return Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      ));
    };

    let mut confirmations = None;

    'outer: for (height, hash) in state.hashes.iter().enumerate() {
      for tx in &state.blocks[hash].txdata {
        if tx.compute_txid() == txid {
          confirmations = Some(state.hashes.len() - height);
          break 'outer;
        }
      }
    }

    Ok(
      serde_json::to_value(GetTransactionResult {
        info: WalletTxInfo {
          txid,
          confirmations: confirmations.unwrap().try_into().unwrap(),
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
    )
  }

  fn get_raw_transaction(
    &self,
    txid: Txid,
    verbose: Option<bool>,
    blockhash: Option<BlockHash>,
  ) -> Result<Value, jsonrpc_core::Error> {
    assert_eq!(blockhash, None, "Blockhash param is unsupported");

    let state = self.state();

    let current_height: u32 = (state.hashes.len() - 1).try_into().unwrap();

    let tx_height = state.txid_to_block_height.get(&txid);

    let confirmations = tx_height.map(|tx_height| current_height - tx_height);

    let blockhash = tx_height.map(|tx_height| state.hashes[usize::try_from(*tx_height).unwrap()]);

    if verbose.unwrap_or(false) {
      match state.transactions.get(&txid) {
        Some(transaction) => Ok(
          serde_json::to_value(GetRawTransactionResult {
            in_active_chain: Some(true),
            hex: Vec::new(),
            txid: Txid::all_zeros(),
            hash: Wtxid::all_zeros(),
            size: 0,
            vsize: 0,
            version: 2,
            locktime: 0,
            vin: Vec::new(),
            vout: transaction
              .output
              .iter()
              .enumerate()
              .map(|(n, output)| GetRawTransactionResultVout {
                n: n.try_into().unwrap(),
                value: output.value,
                script_pub_key: GetRawTransactionResultVoutScriptPubKey {
                  asm: output.script_pubkey.to_asm_string(),
                  hex: output.script_pubkey.clone().into(),
                  req_sigs: None,
                  type_: None,
                  addresses: Vec::new(),
                  address: None,
                },
              })
              .collect(),
            blockhash,
            confirmations,
            time: None,
            blocktime: None,
          })
          .unwrap(),
        ),
        None => Err(Self::not_found()),
      }
    } else {
      match state.transactions.get(&txid) {
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

    let mut unspent = Vec::new();

    for (outpoint, &amount) in &state.utxos {
      if state.locked.contains(outpoint) {
        continue;
      }

      let tx = state.transactions.get(&outpoint.txid).unwrap();

      let tx_out = &tx.output[usize::try_from(outpoint.vout).unwrap()];

      let Ok(address) = Address::from_script(&tx_out.script_pubkey, state.network) else {
        continue;
      };

      if !state.is_wallet_address(&address) {
        continue;
      }

      unspent.push(ListUnspentResultEntry {
        txid: outpoint.txid,
        vout: outpoint.vout,
        address: None,
        label: None,
        redeem_script: None,
        witness_script: None,
        script_pub_key: tx_out.script_pubkey.clone(),
        amount,
        confirmations: 0,
        spendable: true,
        solvable: true,
        descriptor: None,
        safe: true,
      });
    }

    Ok(unspent)
  }

  fn list_lock_unspent(&self) -> Result<Vec<JsonOutPoint>, jsonrpc_core::Error> {
    let state = self.state();
    Ok(
      state
        .locked
        .iter()
        .filter(|outpoint| state.utxos.contains_key(outpoint))
        .map(|outpoint| (*outpoint).into())
        .collect(),
    )
  }

  fn get_raw_change_address(
    &self,
    _address_type: Option<bitcoincore_rpc::json::AddressType>,
  ) -> Result<Address, jsonrpc_core::Error> {
    Ok(self.state().new_address(true))
  }

  fn get_descriptor_info(
    &self,
    desc: String,
  ) -> Result<GetDescriptorInfoResult, jsonrpc_core::Error> {
    Ok(GetDescriptorInfoResult {
      descriptor: desc,
      checksum: None,
      is_range: false,
      is_solvable: false,
      has_private_keys: true,
    })
  }

  fn import_descriptors(
    &self,
    req: Vec<ImportDescriptors>,
  ) -> Result<Vec<ImportMultiResult>, jsonrpc_core::Error> {
    self.state().descriptors.extend(
      req
        .into_iter()
        .map(|params| (params.descriptor, params.timestamp)),
    );

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
  ) -> Result<Address, jsonrpc_core::Error> {
    Ok(self.state().new_address(false))
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
        .chain(state.mempool.iter().map(|tx| (tx.compute_txid(), tx)))
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
      assert!(state.locked.insert(output));
    }

    Ok(true)
  }

  fn list_descriptors(
    &self,
    _with_private_keys: Option<bool>,
  ) -> Result<ListDescriptorsResult, jsonrpc_core::Error> {
    Ok(ListDescriptorsResult {
      wallet_name: "ord".into(),
      descriptors: self
        .state()
        .descriptors
        .iter()
        .map(|(desc, timestamp)| Descriptor {
          desc: desc.to_string(),
          timestamp: *timestamp,
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

  fn list_wallet_dir(&self) -> Result<ListWalletDirResult, jsonrpc_core::Error> {
    Ok(ListWalletDirResult {
      wallets: self
        .list_wallets()?
        .into_iter()
        .map(|name| ListWalletDirItem { name })
        .collect(),
    })
  }

  fn wallet_process_psbt(
    &self,
    psbt: String,
    sign: Option<bool>,
    sighash_type: Option<()>,
    bip32derivs: Option<bool>,
  ) -> Result<WalletProcessPsbtResult, jsonrpc_core::Error> {
    assert!(sighash_type.is_none());
    assert!(bip32derivs.is_none());

    let mut psbt = Psbt::deserialize(
      &base64::engine::general_purpose::STANDARD
        .decode(psbt)
        .unwrap(),
    )
    .unwrap();

    for (i, txin) in psbt.unsigned_tx.input.iter().enumerate() {
      psbt.inputs[i].witness_utxo = Some(
        self
          .state()
          .transactions
          .get(&txin.previous_output.txid)
          .unwrap()
          .output[txin.previous_output.vout as usize]
          .clone(),
      );
    }

    if let Some(sign) = sign {
      if sign {
        for input in psbt.inputs.iter_mut() {
          input.final_script_witness = Some(Witness::from_slice(&[&[0; 64]]));
        }
      }
    }

    Ok(WalletProcessPsbtResult {
      psbt: base64::engine::general_purpose::STANDARD.encode(psbt.serialize()),
      complete: false,
    })
  }

  fn finalize_psbt(
    &self,
    psbt: String,
    _extract: Option<bool>,
  ) -> Result<FinalizePsbtResult, jsonrpc_core::Error> {
    let mut transaction = Psbt::deserialize(
      &base64::engine::general_purpose::STANDARD
        .decode(psbt)
        .unwrap(),
    )
    .unwrap()
    .unsigned_tx;

    for input in &mut transaction.input {
      if input.witness.is_empty() {
        input.witness = Witness::from_slice(&[&[0; 64]]);
      }
    }

    Ok(FinalizePsbtResult {
      psbt: None,
      hex: Some(serialize(&transaction)),
      complete: true,
    })
  }
}
