use {
  bitcoin::{
    blockdata::constants::COIN_VALUE, blockdata::script, hash_types::BlockHash, Block, BlockHeader,
    Network, OutPoint, Transaction, TxIn, TxOut, Txid, Witness,
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
    let nonce = self.nonce;
    self.nonce += 1;
    let mut block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: BlockHash::default(),
        merkle_root: Default::default(),
        time: 0,
        bits: 0,
        nonce,
      },
      txdata: vec![Transaction {
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
          value: 50 * COIN_VALUE,
          script_pubkey: script::Builder::new().into_script(),
        }],
      }],
    };

    block.header.prev_blockhash = *self.hashes.last().unwrap();
    block.txdata.append(&mut self.mempool);

    let block_hash = block.block_hash();
    self.hashes.push(block_hash);
    self.blocks.insert(block_hash, block.clone());
    for tx in &block.txdata {
      self.transactions.insert(tx.txid(), tx.clone());
    }

    block
  }

  fn pop_block(&mut self) -> BlockHash {
    let blockhash = self.hashes.pop().unwrap();
    self.blocks.remove(&blockhash);

    blockhash
  }

  fn broadcast_tx(&mut self, tx: Transaction) {
    self.mempool.push(tx);
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

  pub fn broadcast_dummy_tx(&self) -> Txid {
    let tx = Transaction {
      version: 1,
      lock_time: 0,
      input: Vec::new(),
      output: Vec::new(),
    };
    let txid = tx.txid();
    self.state.lock().unwrap().broadcast_tx(tx);

    txid
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
