use {
  super::*,
  bitcoin::{blockdata::constants::COIN_VALUE, blockdata::script, BlockHeader, TxIn, Witness},
  jsonrpc_core::IoHandler,
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  std::collections::BTreeMap,
};

pub(crate) use tempfile::TempDir;

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}

struct BitcoinRpcData {
  hashes: Vec<BlockHash>,
  blocks: BTreeMap<BlockHash, Block>,
  transactions: BTreeMap<Txid, Transaction>,
  mempool: Vec<Transaction>,
}

impl BitcoinRpcData {
  fn new() -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(Network::Bitcoin);
    let genesis_block_hash = genesis_block.block_hash();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    Self {
      hashes,
      blocks,
      transactions: BTreeMap::new(),
      mempool: Vec::new(),
    }
  }

  fn push_block(&mut self, header: BlockHeader) -> Block {
    let mut block = Block {
      header,
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

  fn broadcast_tx(&mut self, tx: Transaction) {
    self.mempool.push(tx);
  }
}

pub struct BitcoinRpcServer {
  data: Arc<Mutex<BitcoinRpcData>>,
}

impl BitcoinRpcServer {
  fn new() -> Self {
    Self {
      data: Arc::new(Mutex::new(BitcoinRpcData::new())),
    }
  }

  pub(crate) fn spawn() -> BitcoinRpcServerHandle {
    let bitcoin_rpc_server = BitcoinRpcServer::new();
    let data = bitcoin_rpc_server.data.clone();
    let mut io = IoHandler::default();
    io.extend_with(bitcoin_rpc_server.to_delegate());

    let rpc_server = ServerBuilder::new(io)
      .threads(1)
      .start_http(&"127.0.0.1:0".parse().unwrap())
      .unwrap();

    let close_handle = rpc_server.close_handle();
    let port = rpc_server.address().port();

    thread::spawn(|| rpc_server.wait());

    BitcoinRpcServerHandle {
      close_handle: Some(close_handle),
      port,
      data,
    }
  }
}

#[jsonrpc_derive::rpc]
pub trait BitcoinRpcApi {
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

impl BitcoinRpcApi for BitcoinRpcServer {
  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.data.lock().unwrap().hashes.get(height) {
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
    match self.data.lock().unwrap().blocks.get(&block_hash) {
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
    match self.data.lock().unwrap().blocks.get(&block_hash) {
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
    match self.data.lock().unwrap().transactions.get(&txid) {
      Some(tx) => Ok(hex::encode(bitcoin::consensus::encode::serialize(tx))),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }
}

pub(crate) struct BitcoinRpcServerHandle {
  pub(crate) port: u16,
  close_handle: Option<CloseHandle>,
  data: Arc<Mutex<BitcoinRpcData>>,
}

impl BitcoinRpcServerHandle {
  pub(crate) fn url(&self) -> String {
    format!("http://127.0.0.1:{}", self.port)
  }

  pub(crate) fn mine_blocks(&self, num: u64) -> Vec<Block> {
    let mut mined_blocks = Vec::new();
    let mut bitcoin_rpc_data = self.data.lock().unwrap();
    for _ in 0..num {
      let block = bitcoin_rpc_data.push_block(BlockHeader {
        version: 0,
        prev_blockhash: BlockHash::default(),
        merkle_root: Default::default(),
        time: 0,
        bits: 0,
        nonce: 0,
      });
      mined_blocks.push(block);
    }
    mined_blocks
  }

  pub(crate) fn broadcast_dummy_tx(&self) -> Txid {
    let tx = Transaction {
      version: 1,
      lock_time: 0,
      input: Vec::new(),
      output: Vec::new(),
    };
    let txid = tx.txid();
    self.data.lock().unwrap().broadcast_tx(tx);

    txid
  }
}

impl Drop for BitcoinRpcServerHandle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
