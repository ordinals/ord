use {
  bitcoin::{
    blockdata::constants::COIN_VALUE, blockdata::script, hash_types::BlockHash, Block, BlockHeader,
    Network, OutPoint, Script, Transaction, TxIn, TxOut, Txid, Witness,
  },
  jsonrpc_core::IoHandler,
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  std::collections::BTreeMap,
  std::{
    sync::{Arc, Mutex},
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

pub struct TransactionOptions<'a> {
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
  utxos: BTreeMap<OutPoint, u64>,
}

impl State {
  fn new() -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();
    let mut utxos = BTreeMap::new();

    // TODO: should genesis be spendable? we have tests that spend it
    let genesis_block = bitcoin::blockdata::constants::genesis_block(Network::Bitcoin);
    let genesis_block_hash = genesis_block.block_hash();
    let genesis_block_coinbase = genesis_block.txdata[0].clone();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);
    utxos.insert(
      OutPoint::new(genesis_block_coinbase.txid(), 0),
      50 * COIN_VALUE,
    );

    Self {
      blocks,
      hashes,
      mempool: Vec::new(),
      nonce: 0,
      transactions: BTreeMap::new(),
      utxos,
    }
  }

  fn create_utxos(&mut self, transaction: &Transaction) -> u64 {
    let mut total_value = 0;
    for (idx, output) in transaction.output.iter().enumerate() {
      self
        .utxos
        .insert(OutPoint::new(transaction.txid(), idx as u32), output.value);
      total_value += output.value;
    }

    total_value
  }

  fn destroy_utxos(&mut self, transaction: &Transaction) -> u64 {
    let mut total_value = 0;
    for input in transaction.input.iter() {
      match self.utxos.remove(&input.previous_output) {
        Some(value) => total_value += value,
        None => continue,
      };
    }

    total_value
  }

  fn process_mempool(&mut self) -> (u64, Vec<Transaction>) {
    let mut total_fees = 0;
    let transactions = self.mempool.clone();
    self.mempool = Vec::new();
    for tx in transactions.iter() {
      self.transactions.insert(tx.txid(), tx.clone());
      let total_output_value = self.create_utxos(tx);
      let total_input_value = self.destroy_utxos(tx);
      total_fees += total_input_value - total_output_value;
    }

    (total_fees, transactions)
  }

  fn create_coinbase(&mut self, fees: u64) -> Transaction {
    let coinbase = Transaction {
      version: 0,
      lock_time: 0,
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: script::Builder::new()
          .push_scriptint(self.blocks.len().try_into().unwrap())
          .into_script(),
        sequence: 0,
        witness: Witness::new(),
      }],
      output: vec![TxOut {
        value: 50 * COIN_VALUE + fees,
        script_pubkey: Script::new(),
      }],
    };
    self.create_utxos(&coinbase);
    self.destroy_utxos(&coinbase);
    self.transactions.insert(coinbase.txid(), coinbase.clone());

    coinbase
  }

  fn push_block(&mut self) -> Block {
    let (total_fees, mut transactions) = self.process_mempool();
    transactions.insert(0, self.create_coinbase(total_fees));

    let block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: *self.hashes.last().unwrap(),
        merkle_root: Default::default(),
        time: 0,
        bits: 0,
        nonce: self.nonce,
      },
      txdata: transactions,
    };

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

  fn broadcast_tx(&mut self, options: TransactionOptions) -> Txid {
    let mut total_value = 0;
    let inputs: Vec<TxIn> = options
      .input_slots
      .iter()
      .map(|slot| {
        let (block_height, tx_idx, vout) = slot;
        let input_block = self.blocks.get(&self.hashes[*block_height]).unwrap();
        let tx = &input_block.txdata[*tx_idx];
        total_value += tx.output[*vout].value;
        TxIn {
          previous_output: OutPoint::new(tx.txid(), *vout as u32),
          script_sig: Script::new(),
          sequence: 0,
          witness: Witness::new(),
        }
      })
      .collect();

    let value_per_output = (total_value - options.fee) / options.output_count as u64;

    let outputs: Vec<TxOut> = (0..options.output_count)
      .map(|_| TxOut {
        value: value_per_output,
        script_pubkey: script::Builder::new().into_script(),
      })
      .collect();

    let tx = Transaction {
      version: 0,
      lock_time: 0,
      input: inputs,
      output: outputs,
    };
    self.mempool.push(tx.clone());

    tx.txid()
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
}

#[jsonrpc_derive::rpc]
pub trait Api {
  #[rpc(name = "getblockhash")]
  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error>;

  #[rpc(name = "getblockheader")]
  fn getblockheader(
    &self,
    blockhash: BlockHash,
    verbose: bool,
  ) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "getblock")]
  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "getrawtransaction")]
  fn get_raw_transaction(
    &self,
    txid: Txid,
    verbose: bool,
    blockhash: Option<BlockHash>,
  ) -> Result<String, jsonrpc_core::Error>;
}

impl Api for Server {
  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.state.lock().unwrap().hashes.get(height) {
      Some(block_hash) => Ok(*block_hash),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn getblockheader(
    &self,
    block_hash: BlockHash,
    verbose: bool,
  ) -> Result<String, jsonrpc_core::Error> {
    assert!(!verbose);
    match self.state.lock().unwrap().blocks.get(&block_hash) {
      Some(block) => Ok(hex::encode(bitcoin::consensus::encode::serialize(
        &block.header,
      ))),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn getblock(&self, block_hash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error> {
    assert_eq!(verbosity, 0, "Verbosity level {verbosity} is unsupported");
    match self.state.lock().unwrap().blocks.get(&block_hash) {
      Some(block) => Ok(hex::encode(bitcoin::consensus::encode::serialize(block))),
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
  ) -> Result<String, jsonrpc_core::Error> {
    assert!(!verbose, "Verbose param is unsupported");
    assert_eq!(blockhash, None, "Blockhash param is unsupported");
    match self.state.lock().unwrap().transactions.get(&txid) {
      Some(tx) => Ok(hex::encode(bitcoin::consensus::encode::serialize(tx))),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
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

  pub fn mine_blocks(&self, num: u64) -> Vec<Block> {
    let mut bitcoin_rpc_data = self.state.lock().unwrap();
    (0..num).map(|_| bitcoin_rpc_data.push_block()).collect()
  }

  pub fn broadcast_tx(&self, options: TransactionOptions) -> Txid {
    self.state.lock().unwrap().broadcast_tx(options)
  }

  pub fn invalidate_tip(&self) -> BlockHash {
    self.state.lock().unwrap().pop_block()
  }
}

impl Drop for Handle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
