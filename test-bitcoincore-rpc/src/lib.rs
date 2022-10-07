use {
  bitcoin::{
    blockdata::constants::COIN_VALUE,
    blockdata::script,
    consensus::encode::{deserialize, serialize},
    hash_types::BlockHash,
    hashes::Hash,
    util::amount::SignedAmount,
    Amount, Block, BlockHeader, Network, OutPoint, PackedLockTime, Script, Sequence, Transaction,
    TxIn, TxMerkleNode, TxOut, Txid, Witness, Wtxid,
  },
  bitcoincore_rpc::json::{
    Bip125Replaceable, CreateRawTransactionInput, GetBlockHeaderResult, GetBlockchainInfoResult,
    GetNetworkInfoResult, GetRawTransactionResult, GetTransactionResult, ListUnspentResultEntry,
    SignRawTransactionResult, WalletTxInfo,
  },
  jsonrpc_core::{IoHandler, Value},
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  std::collections::BTreeMap,
  std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    thread,
  },
};

pub fn spawn() -> Handle {
  let server = Server::new();
  let state = server.state.clone();
  let mut io = IoHandler::default();
  io.extend_with(server.to_delegate());

  let rpc_server = ServerBuilder::new(io)
    .threads(1)
    .start_http(&"127.0.0.1:0".parse().unwrap())
    .unwrap();

  let close_handle = rpc_server.close_handle();
  let port = rpc_server.address().port();

  thread::spawn(|| rpc_server.wait());

  Handle {
    close_handle: Some(close_handle),
    port,
    state,
  }
}

pub struct TransactionTemplate<'a> {
  pub input_slots: &'a [(usize, usize, usize)],
  pub output_count: usize,
  pub fee: u64,
}

struct State {
  blocks: BTreeMap<BlockHash, Block>,
  hashes: Vec<BlockHash>,
  mempool: Vec<Transaction>,
  nonce: u32,
  transactions: BTreeMap<Txid, Transaction>,
}

impl State {
  fn new() -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(Network::Bitcoin);
    let genesis_block_hash = genesis_block.block_hash();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    Self {
      blocks,
      hashes,
      mempool: Vec::new(),
      nonce: 0,
      transactions: BTreeMap::new(),
    }
  }

  fn push_block(&mut self) -> Block {
    let coinbase = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: script::Builder::new()
          .push_scriptint(self.blocks.len().try_into().unwrap())
          .into_script(),
        sequence: Sequence(0),
        witness: Witness::new(),
      }],
      output: vec![TxOut {
        value: 50 * COIN_VALUE
          + self
            .mempool
            .iter()
            .map(|tx| {
              tx.input
                .iter()
                .map(|txin| {
                  self.transactions[&txin.previous_output.txid].output
                    [txin.previous_output.vout as usize]
                    .value
                })
                .sum::<u64>()
                - tx.output.iter().map(|txout| txout.value).sum::<u64>()
            })
            .sum::<u64>(),
        script_pubkey: Script::new(),
      }],
    };

    let block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: *self.hashes.last().unwrap(),
        merkle_root: TxMerkleNode::all_zeros(),
        time: 0,
        bits: 0,
        nonce: self.nonce,
      },
      txdata: std::iter::once(coinbase)
        .chain(self.mempool.drain(0..))
        .collect(),
    };

    for tx in &block.txdata {
      self.transactions.insert(tx.txid(), tx.clone());
    }
    self.blocks.insert(block.block_hash(), block.clone());
    self.hashes.push(block.block_hash());
    self.nonce += 1;

    block
  }

  fn pop_block(&mut self) -> BlockHash {
    let blockhash = self.hashes.pop().unwrap();
    self.blocks.remove(&blockhash);

    blockhash
  }

  fn broadcast_tx(&mut self, options: TransactionTemplate) -> Txid {
    let mut total_value = 0;
    let mut input = Vec::new();
    for (height, tx, vout) in options.input_slots {
      let tx = &self.blocks.get(&self.hashes[*height]).unwrap().txdata[*tx];
      total_value += tx.output[*vout].value;
      input.push(TxIn {
        previous_output: OutPoint::new(tx.txid(), *vout as u32),
        script_sig: Script::new(),
        sequence: Sequence(0),
        witness: Witness::new(),
      });
    }

    let value_per_output = (total_value - options.fee) / options.output_count as u64;
    assert_eq!(
      value_per_output * options.output_count as u64 + options.fee,
      total_value
    );

    let tx = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input,
      output: (0..options.output_count)
        .map(|_| TxOut {
          value: value_per_output,
          script_pubkey: script::Builder::new().into_script(),
        })
        .collect(),
    };
    self.mempool.push(tx.clone());

    tx.txid()
  }

  fn get_mempool(&self) -> Vec<Transaction> {
    self.mempool.clone()
  }
}

pub struct Server {
  state: Arc<Mutex<State>>,
}

impl Server {
  fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(State::new())),
    }
  }

  fn state(&self) -> MutexGuard<State> {
    self.state.lock().unwrap()
  }

  fn not_found() -> jsonrpc_core::Error {
    jsonrpc_core::Error::new(jsonrpc_core::types::error::ErrorCode::ServerError(-8))
  }
}

#[jsonrpc_derive::rpc]
pub trait Api {
  #[rpc(name = "getblockchaininfo")]
  fn get_blockchain_info(&self) -> Result<GetBlockchainInfoResult, jsonrpc_core::Error>;

  #[rpc(name = "getnetworkinfo")]
  fn get_network_info(&self) -> Result<GetNetworkInfoResult, jsonrpc_core::Error>;

  #[rpc(name = "getblockhash")]
  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error>;

  #[rpc(name = "getblockheader")]
  fn getblockheader(
    &self,
    block_hash: BlockHash,
    verbose: bool,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "getblock")]
  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "createrawtransaction")]
  fn create_raw_transaction(
    &self,
    utxos: Vec<CreateRawTransactionInput>,
    outs: HashMap<String, f64>,
    locktime: Option<i64>,
    replaceable: Option<bool>,
  ) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "signrawtransactionwithwallet")]
  fn sign_raw_transaction_with_wallet(
    &self,
    tx: String,
    utxos: Option<()>,
    sighash_type: Option<()>,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "sendrawtransaction")]
  fn send_raw_transaction(&self, tx: String) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "gettransaction")]
  fn get_transaction(
    &self,
    txid: Txid,
    include_watchonly: Option<bool>,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "getrawtransaction")]
  fn get_raw_transaction(
    &self,
    txid: Txid,
    verbose: bool,
    blockhash: Option<BlockHash>,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "listunspent")]
  fn list_unspent(
    &self,
    minconf: Option<usize>,
    maxconf: Option<usize>,
    address: Option<bitcoin::Address>,
    include_unsafe: Option<bool>,
    query_options: Option<String>,
  ) -> Result<Vec<ListUnspentResultEntry>, jsonrpc_core::Error>;
}

impl Api for Server {
  fn get_blockchain_info(&self) -> Result<GetBlockchainInfoResult, jsonrpc_core::Error> {
    Ok(GetBlockchainInfoResult {
      chain: String::from("test-bitcoincore-rpc"),
      blocks: 0,
      headers: 0,
      best_block_hash: self.state.lock().unwrap().hashes[0],
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
      warnings: String::from(""),
    })
  }

  fn get_network_info(&self) -> Result<GetNetworkInfoResult, jsonrpc_core::Error> {
    Ok(GetNetworkInfoResult {
      version: 230000,
      subversion: String::from(""),
      protocol_version: 0,
      local_services: String::from(""),
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
      warnings: String::from(""),
    })
  }

  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.state().hashes.get(height) {
      Some(block_hash) => Ok(*block_hash),
      None => Err(Self::not_found()),
    }
  }

  fn getblockheader(
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

  fn getblock(&self, block_hash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error> {
    assert_eq!(verbosity, 0, "Verbosity level {verbosity} is unsupported");
    match self.state().blocks.get(&block_hash) {
      Some(block) => Ok(hex::encode(serialize(block))),
      None => Err(Self::not_found()),
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
      lock_time: PackedLockTime(0),
      input: utxos
        .iter()
        .map(|input| TxIn {
          previous_output: OutPoint::new(input.txid, input.vout),
          script_sig: Script::new(),
          sequence: Sequence(0),
          witness: Witness::new(),
        })
        .collect(),
      output: outs
        .iter()
        .map(|(_address, amount)| TxOut {
          value: *amount as u64,
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
        .transactions
        .iter()
        .flat_map(|(txid, tx)| {
          (0..tx.output.len()).map(|vout| ListUnspentResultEntry {
            txid: *txid,
            vout: vout as u32,
            address: None,
            label: None,
            redeem_script: None,
            witness_script: None,
            script_pub_key: Script::new(),
            amount: Amount::default(),
            confirmations: 0,
            spendable: true,
            solvable: true,
            descriptor: None,
            safe: true,
          })
        })
        .collect(),
    )
  }
}

pub struct Handle {
  close_handle: Option<CloseHandle>,
  port: u16,
  state: Arc<Mutex<State>>,
}

impl Handle {
  pub fn url(&self) -> String {
    format!("http://127.0.0.1:{}", self.port)
  }

  fn state(&self) -> MutexGuard<State> {
    self.state.lock().unwrap()
  }

  pub fn mine_blocks(&self, num: u64) -> Vec<Block> {
    let mut bitcoin_rpc_data = self.state.lock().unwrap();
    (0..num).map(|_| bitcoin_rpc_data.push_block()).collect()
  }

  pub fn broadcast_tx(&self, options: TransactionTemplate) -> Txid {
    self.state().broadcast_tx(options)
  }

  pub fn invalidate_tip(&self) -> BlockHash {
    self.state().pop_block()
  }

  pub fn tx(&self, bi: usize, ti: usize) -> Transaction {
    let state = self.state();
    state.blocks[&state.hashes[bi]].txdata[ti].clone()
  }

  pub fn mempool(&self) -> Vec<Transaction> {
    self.state.lock().unwrap().get_mempool()
  }
}

impl Drop for Handle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
