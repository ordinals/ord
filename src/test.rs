use {
  super::*,
  bitcoin::BlockHeader,
  bitcoincore_rpc::Auth,
  jsonrpc_core::IoHandler,
  jsonrpc_derive::rpc,
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  std::collections::BTreeMap,
};

pub(crate) use {bitcoincore_rpc::RpcApi, tempfile::TempDir};


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

struct Blocks {
  hashes: Vec<BlockHash>,
  blocks: BTreeMap<BlockHash, Block>,
  transactions: BTreeMap<Txid, Transaction>,
  mempool: Vec<Transaction>,
}

impl Blocks {
  fn new() -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();
    let transactions = BTreeMap::new();
    let mempool = Vec::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(Network::Bitcoin);
    let genesis_block_hash = genesis_block.block_hash();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    Self {
      hashes,
      blocks,
      transactions,
      mempool,
    }
  }

  fn push_block(&mut self, header: BlockHeader) -> BlockHash {
    let coinbase = Transaction {
      version: 1,
      lock_time: 0,
      input: Vec::new(),
      output: Vec::new(),
    };
    // This feels like a bad way to do this
    let mut transactions = self.mempool.clone();
    transactions.push(coinbase.clone());
    self.build_tx_index(transactions);

    let mut txdata = vec![coinbase];
    txdata.append(&mut self.mempool);

    let mut block = Block { header, txdata };
    block.header.prev_blockhash = *self.hashes.last().unwrap();
    let block_hash = block.block_hash();
    self.hashes.push(block_hash);
    self.blocks.insert(block_hash, block);
    block_hash
  }

  fn build_tx_index(&mut self, txs: Vec<Transaction>) {
    for tx in txs {
      self.transactions.insert(tx.txid(), tx);
    }
  }

  fn broadcast_tx(&mut self, tx: Transaction) {
    self.mempool.push(tx);
  }
}

pub struct BitcoinRpcServer {
  blocks: Arc<Mutex<Blocks>>,
}

impl BitcoinRpcServer {
  fn new() -> Self {
    Self {
      blocks: Arc::new(Mutex::new(Blocks::new())),
    }
  }

  pub(crate) fn spawn() -> BitcoinRpcServerHandle {
    let bitcoin_rpc_server = BitcoinRpcServer::new();
    let blocks = bitcoin_rpc_server.blocks.clone();
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
      blocks,
    }
  }
}

#[rpc(server)]
pub trait BitcoinRpc {
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

  #[rpc(name = "generatetoaddress")]
  fn generate_to_address(
    &self,
    count: usize,
    address: Address,
  ) -> Result<Vec<bitcoin::BlockHash>, jsonrpc_core::Error>;

  #[rpc(name = "getrawtransaction")]
  fn get_raw_transaction(
    &self,
    txid: String, // why doesn't Txid work? Do I need to implement a from<String> trait for Txid?
    verbose: bool,
    blockhash: Option<BlockHash>,
  ) -> Result<Transaction, jsonrpc_core::Error>;
}

impl BitcoinRpc for BitcoinRpcServer {
  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.blocks.lock().unwrap().hashes.get(height) {
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
    match self.blocks.lock().unwrap().blocks.get(&block_hash) {
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
    match self.blocks.lock().unwrap().blocks.get(&block_hash) {
      Some(block) => Ok(hex::encode(bitcoin::consensus::encode::serialize(block))),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn generate_to_address(
    &self,
    count: usize,
    _address: Address,
  ) -> Result<Vec<BlockHash>, jsonrpc_core::Error> {
    let mut block_hashes = Vec::new();
    let mut blocks = self.blocks.lock().unwrap();

    for _ in 0..count {
      block_hashes.push(blocks.push_block(BlockHeader {
        version: 0,
        prev_blockhash: BlockHash::default(),
        merkle_root: Default::default(),
        time: 0,
        bits: 0,
        nonce: 0,
      }));
    }

    Ok(block_hashes)
  }

  fn get_raw_transaction(
    &self,
    txid: String,
    verbose: bool,
    blockhash: Option<BlockHash>,
  ) -> Result<Transaction, jsonrpc_core::Error> {
    // TODO: correct way of doing this?
    assert_eq!(verbose, false, "Verbose param is unsupported");
    assert_eq!(blockhash, None, "Blockhash param is unsupported");

    let txid_hash = bitcoin::hashes::sha256d::Hash::from_str(&txid).unwrap();
    let txid = Txid::from_hash(txid_hash);
    match self.blocks.lock().unwrap().transactions.get(&txid) {
      Some(tx) => Ok(tx.clone()),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }
}

pub(crate) struct BitcoinRpcServerHandle {
  pub(crate) port: u16,
  close_handle: Option<CloseHandle>,
  blocks: Arc<Mutex<Blocks>>,
}

impl BitcoinRpcServerHandle {
  pub(crate) fn url(&self) -> String {
    format!("http://127.0.0.1:{}", self.port)
  }

  pub(crate) fn client(&self) -> bitcoincore_rpc::Client {
    bitcoincore_rpc::Client::new(&self.url(), Auth::None).unwrap()
  }

  pub(crate) fn mine_blocks(&self, num: u64) -> Vec<BlockHash> {
    self
      .client()
      .generate_to_address(num, &"1BitcoinEaterAddressDontSendf59kuE".parse().unwrap())
      .unwrap()
  }

  pub(crate) fn broadcast_dummy_tx(&self) -> Txid {
    let tx = Transaction {
      version: 1,
      lock_time: 0,
      input: Vec::new(),
      output: Vec::new(),
    };
    let txid = tx.txid();
    self.blocks.lock().unwrap().broadcast_tx(tx);

    txid
  }
}

impl Drop for BitcoinRpcServerHandle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
