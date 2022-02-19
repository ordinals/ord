use {
  super::*, jsonrpc_core::IoHandler, jsonrpc_core::Result, jsonrpc_derive::rpc,
  jsonrpc_http_server::CloseHandle, jsonrpc_http_server::ServerBuilder,
};

#[rpc]
pub trait RpcApi {
  #[rpc(name = "getblockhash")]
  fn getblockhash(&self, height: usize) -> Result<BlockHash>;

  #[rpc(name = "getblock")]
  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String>;
}

pub struct RpcServer {
  blocks: Vec<Block>,
}

impl RpcServer {
  pub(crate) fn spawn(blocks: &[Block]) -> (CloseHandle, u16) {
    let server = Self {
      blocks: blocks.to_vec(),
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

    (close_handle, port)
  }
}

impl RpcApi for RpcServer {
  fn getblockhash(&self, height: usize) -> Result<BlockHash> {
    match self.blocks.get(height) {
      Some(block) => Ok(block.block_hash()),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }
  }

  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String> {
    assert_eq!(verbosity, 0, "Verbosity level {verbosity} is unsupported");

    for block in &self.blocks {
      if block.block_hash() == blockhash {
        let mut encoded = Vec::new();
        block.consensus_encode(&mut encoded).unwrap();
        return Ok(hex::encode(encoded));
      }
    }

    panic!("No block with hash {blockhash}")
  }
}
