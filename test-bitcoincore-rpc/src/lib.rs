use {
  api::Api,
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
  server::Server,
  state::State,
  std::collections::BTreeMap,
  std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    thread,
  },
};

mod api;
mod server;
mod state;

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
    self.state.lock().unwrap().mempool().to_vec()
  }
}

impl Drop for Handle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
