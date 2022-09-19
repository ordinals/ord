use {
  super::*,
  bitcoin::BlockHeader,
  jsonrpc_core::IoHandler,
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  std::collections::BTreeMap,
};

pub(crate) use {regex::Regex, tempfile::TempDir};

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

pub struct BitcoinRpcServer {
  block_hashes: Vec<BlockHash>,
  blocks: BTreeMap<BlockHash, Block>,
}

impl BitcoinRpcServer {
  fn new() -> Self {
    let mut blocks = BTreeMap::new();
    let mut block_hashes = Vec::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(Network::Bitcoin);
    let genesis_block_hash = genesis_block.block_hash();

    block_hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    let next = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: genesis_block_hash,
        merkle_root: Default::default(),
        time: 0,
        bits: 0,
        nonce: 0,
      },
      txdata: Vec::new(),
    };

    let next_block_hash = next.block_hash();

    block_hashes.push(next_block_hash);
    blocks.insert(next_block_hash, next);

    Self {
      block_hashes,
      blocks,
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

  #[rpc(name = "getblock")]
  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error>;
}

impl BitcoinRpc for BitcoinRpcServer {
  fn getblockhash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error> {
    match self.block_hashes.get(height) {
      Some(block_hash) => Ok(*block_hash),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn getblock(&self, block_hash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error> {
    assert_eq!(verbosity, 0, "Verbosity level {verbosity} is unsupported");
    match self.blocks.get(&block_hash) {
      Some(block) => Ok(hex::encode(bitcoin::consensus::encode::serialize(block))),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }
}

pub(crate) struct BitcoinRpcServerHandle {
  pub(crate) port: u16,
  close_handle: Option<CloseHandle>,
}

impl Drop for BitcoinRpcServerHandle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
