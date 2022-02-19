use super::Block;
use bitcoin::{consensus::Encodable, BlockHash};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

#[rpc]
pub trait Rpc {
  #[rpc(name = "getblockhash")]
  fn getblockhash(&self, height: usize) -> Result<BlockHash>;

  #[rpc(name = "getblock")]
  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String>;
}

pub struct Server {
  pub blocks: Vec<Block>,
}

impl Rpc for Server {
  fn getblockhash(&self, height: usize) -> Result<BlockHash> {
    match self.blocks.get(height) {
      Some(block) => Ok(block.block_hash()),
      None => Err(jsonrpc_core::Error::new(
        jsonrpc_core::types::error::ErrorCode::ServerError(-8),
      )),
    }

    // if height + 1 > self.blocks.len() {
    //   return Err("bad!");
    // }

    // Ok(self.blocks[height as usize].block_hash())
  }

  fn getblock(&self, blockhash: BlockHash, verbosity: u64) -> Result<String> {
    assert_eq!(verbosity, 0);

    for block in &self.blocks {
      if block.block_hash() == blockhash {
        let mut encoded = Vec::new();
        block.consensus_encode(&mut encoded).unwrap();
        return Ok(hex::encode(encoded));
      }
    }

    panic!()
  }
}
