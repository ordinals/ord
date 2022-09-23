use {
  super::*,
  bitcoin::BlockHeader,
  bitcoincore_rpc::Auth,
  jsonrpc_core::IoHandler,
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  std::collections::BTreeMap,
};

pub(crate) use {bitcoincore_rpc::RpcApi, tempfile::TempDir};

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let pattern: &'static str = $pattern;
    let regex = Regex::new(&format!("^(?s){}$", pattern)).unwrap();
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
}

impl Blocks {
  fn new() -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(Network::Bitcoin);
    let genesis_block_hash = genesis_block.block_hash();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    Self { hashes, blocks }
  }

  fn push_block(&mut self, mut block: Block) -> BlockHash {
    block.header.prev_blockhash = *self.hashes.last().unwrap();
    let block_hash = block.block_hash();
    self.hashes.push(block_hash);
    self.blocks.insert(block_hash, block);
    block_hash
  }
}

pub struct BitcoinRpcServer {
  blocks: Mutex<Blocks>,
}

impl BitcoinRpcServer {
  fn new() -> Self {
    Self {
      blocks: Mutex::new(Blocks::new()),
    }
  }

  pub(crate) fn spawn() -> BitcoinRpcServerHandle {
    let mut io = IoHandler::default();
    io.extend_with(BitcoinRpcServer::new().to_delegate());

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
    }
  }
}

#[jsonrpc_derive::rpc]
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
      block_hashes.push(blocks.push_block(Block {
        header: BlockHeader {
          version: 0,
          prev_blockhash: BlockHash::default(),
          merkle_root: Default::default(),
          time: 0,
          bits: 0,
          nonce: 0,
        },
        txdata: Vec::new(),
      }));
    }

    Ok(block_hashes)
  }
}

pub(crate) struct BitcoinRpcServerHandle {
  pub(crate) port: u16,
  close_handle: Option<CloseHandle>,
}

impl BitcoinRpcServerHandle {
  pub(crate) fn url(&self) -> String {
    format!("http://127.0.0.1:{}", self.port)
  }

  pub(crate) fn client(&self) -> bitcoincore_rpc::Client {
    bitcoincore_rpc::Client::new(&self.url(), Auth::None).unwrap()
  }

  pub(crate) fn mine_blocks(&self, num: u64) {
    self
      .client()
      .generate_to_address(num, &"1BitcoinEaterAddressDontSendf59kuE".parse().unwrap())
      .unwrap();
  }
}

impl Drop for BitcoinRpcServerHandle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
