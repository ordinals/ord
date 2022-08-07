use {
  super::*,
  bitcoin::{hash_types::BlockHash, Txid},
  jsonrpc_core::{IoHandler, Result},
  jsonrpc_derive::rpc,
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
};

#[rpc]
pub trait RpcApi {
  #[rpc(name = "getblockhash")]
  fn getblockhash(&self, height: usize) -> Result<BlockHash>;

  #[rpc(name = "getblock")]
  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String>;

  #[rpc(name = "getrawtransaction")]
  fn getrawtransaction(
    &self,
    txid: Txid,
    verbose: bool,
    block_hash: Option<BlockHash>,
  ) -> Result<String>;
}

pub struct RpcServer {
  blocks: Arc<Mutex<Vec<Block>>>,
  calls: Arc<Mutex<Vec<String>>>,
}

impl RpcServer {
  pub(crate) fn spawn(
    blocks: Vec<Block>,
  ) -> (
    Arc<Mutex<Vec<Block>>>,
    CloseHandle,
    Arc<Mutex<Vec<String>>>,
    u16,
  ) {
    let calls = Arc::new(Mutex::new(Vec::new()));

    let blocks = Arc::new(Mutex::new(blocks));

    let server = Self {
      blocks: blocks.clone(),
      calls: calls.clone(),
    };

    let mut io = IoHandler::default();
    io.extend_with(server.to_delegate());

    let server = ServerBuilder::new(io)
      .threads(1)
      .start_http(&"127.0.0.1:0".parse().unwrap())
      .unwrap();

    let close_handle = server.close_handle();

    let port = server.address().port();

    thread::spawn(|| server.wait());

    (blocks, close_handle, calls, port)
  }

  fn call(&self, method: &str) {
    self.calls.lock().unwrap().push(method.into());
  }
}

impl RpcApi for RpcServer {
  fn getblockhash(&self, height: usize) -> Result<BlockHash> {
    self.call("getblockhash");

    match self.blocks.lock().unwrap().get(height) {
      Some(block) => Ok(block.block_hash()),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String> {
    self.call("getblock");

    assert_eq!(verbosity, 0, "Verbosity level {verbosity} is unsupported");

    for block in self.blocks.lock().unwrap().iter() {
      if block.block_hash() == blockhash {
        let mut encoded = Vec::new();
        block.consensus_encode(&mut encoded).unwrap();
        return Ok(hex::encode(encoded));
      }
    }

    Err(jsonrpc_core::Error::new(
      jsonrpc_core::types::error::ErrorCode::ServerError(-8),
    ))
  }

  fn getrawtransaction(
    &self,
    txid: Txid,
    verbose: bool,
    block_hash: Option<BlockHash>,
  ) -> Result<String> {
    self.call("getrawtransaction");

    assert!(!verbose, "Verbose flag {verbose} is unsupported");

    assert_eq!(
      block_hash, None,
      "Passing in a block hash {:?} is unsupported",
      block_hash
    );

    for block in self.blocks.lock().unwrap().iter() {
      for transaction in &block.txdata {
        if transaction.txid() == txid {
          let mut buffer = Vec::new();
          transaction.consensus_encode(&mut buffer).unwrap();
          return Ok(hex::encode(buffer));
        }
      }
    }

    Err(jsonrpc_core::Error::new(
      jsonrpc_core::types::error::ErrorCode::ServerError(-8),
    ))
  }
}
