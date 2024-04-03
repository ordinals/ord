#![allow(clippy::too_many_arguments)]

use {
  api::Api,
  bitcoin::{
    address::{Address, NetworkUnchecked},
    amount::SignedAmount,
    block::Header,
    blockdata::constants::COIN_VALUE,
    blockdata::{block::Version, script},
    consensus::encode::{deserialize, serialize},
    hash_types::{BlockHash, TxMerkleNode},
    hashes::Hash,
    locktime::absolute::LockTime,
    pow::CompactTarget,
    Amount, Block, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    Wtxid,
  },
  bitcoincore_rpc::json::{
    Bip125Replaceable, CreateRawTransactionInput, Descriptor, EstimateMode, FeeRatePercentiles,
    FinalizePsbtResult, GetBalancesResult, GetBalancesResultEntry, GetBlockHeaderResult,
    GetBlockStatsResult, GetBlockchainInfoResult, GetDescriptorInfoResult, GetNetworkInfoResult,
    GetRawTransactionResult, GetRawTransactionResultVout, GetRawTransactionResultVoutScriptPubKey,
    GetTransactionResult, GetTransactionResultDetail, GetTransactionResultDetailCategory,
    GetTxOutResult, GetWalletInfoResult, ImportDescriptors, ImportMultiResult,
    ListDescriptorsResult, ListTransactionResult, ListUnspentResultEntry, ListWalletDirItem,
    ListWalletDirResult, LoadWalletResult, SignRawTransactionInput, SignRawTransactionResult,
    Timestamp, WalletProcessPsbtResult, WalletTxInfo,
  },
  jsonrpc_core::{IoHandler, Value},
  jsonrpc_http_server::{CloseHandle, ServerBuilder},
  serde::{Deserialize, Serialize},
  server::Server,
  state::State,
  std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
};

mod api;
mod server;
mod state;

pub fn builder() -> Builder {
  Builder {
    fail_lock_unspent: false,
    network: Network::Bitcoin,
    version: 240000,
  }
}

pub struct Builder {
  fail_lock_unspent: bool,
  network: Network,
  version: usize,
}

impl Builder {
  pub fn fail_lock_unspent(self, fail_lock_unspent: bool) -> Self {
    Self {
      fail_lock_unspent,
      ..self
    }
  }

  pub fn network(self, network: Network) -> Self {
    Self { network, ..self }
  }

  pub fn version(self, version: usize) -> Self {
    Self { version, ..self }
  }

  pub fn build(self) -> Handle {
    let state = Arc::new(Mutex::new(State::new(
      self.network,
      self.version,
      self.fail_lock_unspent,
    )));
    let server = Server::new(state.clone());
    let mut io = IoHandler::default();
    io.extend_with(server.to_delegate());

    let rpc_server = ServerBuilder::new(io)
      .threads(1)
      .start_http(&"127.0.0.1:0".parse().unwrap())
      .unwrap();

    let close_handle = rpc_server.close_handle();
    let port = rpc_server.address().port();

    thread::spawn(|| rpc_server.wait());

    for i in 0.. {
      match reqwest::blocking::get(format!("http://127.0.0.1:{port}/")) {
        Ok(_) => break,
        Err(err) => {
          if i == 400 {
            panic!("mock bitcoind server failed to start: {err}");
          }
        }
      }

      thread::sleep(Duration::from_millis(25));
    }

    let tempdir = TempDir::new().unwrap();

    fs::write(tempdir.path().join(".cookie"), "username:password").unwrap();

    Handle {
      close_handle: Some(close_handle),
      tempdir,
      port,
      state,
    }
  }
}

pub fn spawn() -> Handle {
  builder().build()
}

#[derive(Clone)]
pub struct TransactionTemplate<'a> {
  pub fee: u64,
  pub inputs: &'a [(usize, usize, usize, Witness)],
  pub op_return: Option<ScriptBuf>,
  pub op_return_index: Option<usize>,
  pub output_values: &'a [u64],
  pub outputs: usize,
  pub p2tr: bool,
}

<<<<<<<< HEAD:crates/test-bitcoincore-rpc/src/lib.rs
#[derive(Serialize, Deserialize)]
========
#[derive(Serialize, Deserialize, Debug)]
>>>>>>>> origin/ordzaar-master-0-17-1:crates/mockcore/src/lib.rs
pub struct JsonOutPoint {
  txid: Txid,
  vout: u32,
}

impl From<OutPoint> for JsonOutPoint {
  fn from(outpoint: OutPoint) -> Self {
    Self {
      txid: outpoint.txid,
      vout: outpoint.vout,
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundRawTransactionOptions {
  #[serde(with = "bitcoin::amount::serde::as_btc::opt")]
  fee_rate: Option<Amount>,
  #[serde(skip_serializing_if = "Option::is_none")]
  change_position: Option<u32>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FundRawTransactionResult {
  #[serde(with = "bitcoincore_rpc::json::serde_hex")]
  pub hex: Vec<u8>,
  #[serde(with = "bitcoin::amount::serde::as_btc")]
  pub fee: Amount,
  #[serde(rename = "changepos")]
  pub change_position: i32,
}

impl<'a> Default for TransactionTemplate<'a> {
  fn default() -> Self {
    Self {
      fee: 0,
      inputs: &[],
      op_return: None,
      op_return_index: None,
      output_values: &[],
      outputs: 1,
      p2tr: false,
    }
  }
}

pub struct Handle {
  close_handle: Option<CloseHandle>,
  port: u16,
  state: Arc<Mutex<State>>,
  tempdir: TempDir,
}

impl Handle {
  pub fn url(&self) -> String {
    format!("http://127.0.0.1:{}", self.port)
  }

  pub fn address(&self, output: OutPoint) -> Address {
    let state = self.state();

    Address::from_script(
      &state.transactions.get(&output.txid).unwrap().output[output.vout as usize].script_pubkey,
      state.network,
    )
    .unwrap()
  }

  pub fn state(&self) -> MutexGuard<State> {
    self.state.lock().unwrap()
  }

  pub fn clear_state(&self) {
    self.state.lock().unwrap().clear();
  }

  pub fn wallets(&self) -> BTreeSet<String> {
    self.state().wallets.clone()
  }

  #[track_caller]
  pub fn mine_blocks(&self, n: u64) -> Vec<Block> {
    self.mine_blocks_with_subsidy(n, 50 * COIN_VALUE)
  }

  #[track_caller]
  pub fn mine_blocks_with_subsidy(&self, n: u64, subsidy: u64) -> Vec<Block> {
    let mut bitcoin_rpc_data = self.state();
    let mut blocks = Vec::new();
    for _ in 0..n {
      blocks.push(bitcoin_rpc_data.mine_block(subsidy));
    }
    blocks
  }

  pub fn broadcast_tx(&self, template: TransactionTemplate) -> Txid {
    self.state().broadcast_tx(template)
  }

  pub fn height(&self) -> u64 {
    u64::try_from(self.state().blocks.len()).unwrap() - 1
  }

  pub fn invalidate_tip(&self) -> BlockHash {
    self.state().pop_block()
  }

  pub fn get_utxo_amount(&self, outpoint: &OutPoint) -> Option<Amount> {
    self.state().utxos.get(outpoint).cloned()
  }

  #[track_caller]
  pub fn tx(&self, block: usize, transaction: usize) -> Transaction {
    let state = self.state();
    let blockhash = state.hashes.get(block).expect("block index out of bounds");
    state.blocks[blockhash]
      .txdata
      .get(transaction)
      .expect("transaction index out of bounds")
      .clone()
  }

  #[track_caller]
  pub fn tx_by_id(&self, txid: Txid) -> Transaction {
    self
      .state()
      .transactions
      .get(&txid)
      .expect("unknown transaction")
      .clone()
  }

  pub fn mempool(&self) -> Vec<Transaction> {
    self.state().mempool().to_vec()
  }

  pub fn descriptors(&self) -> Vec<String> {
    self.state().descriptors.clone()
  }

  pub fn import_descriptor(&self, desc: String) {
    self.state().descriptors.push(desc);
  }

  pub fn lock(&self, output: OutPoint) {
    self.state().locked.insert(output);
  }

  pub fn network(&self) -> String {
    match self.state().network {
      Network::Bitcoin => "mainnet".to_string(),
      Network::Testnet => Network::Testnet.to_string(),
      Network::Signet => Network::Signet.to_string(),
      Network::Regtest => Network::Regtest.to_string(),
      _ => panic!(),
    }
  }

  pub fn loaded_wallets(&self) -> BTreeSet<String> {
    self.state().loaded_wallets.clone()
  }

  pub fn cookie_file(&self) -> PathBuf {
    self.tempdir.path().join(".cookie")
  }

  pub fn get_locked(&self) -> BTreeSet<OutPoint> {
    self.state().get_locked()
  }
}

impl Drop for Handle {
  fn drop(&mut self) {
    self.close_handle.take().unwrap().close();
  }
}
