use super::*;

#[doc(hidden)]
#[jsonrpc_derive::rpc(server)]
pub trait Api {
  #[rpc(name = "getblockchaininfo")]
  fn get_blockchain_info(&self) -> Result<GetBlockchainInfoResult, jsonrpc_core::Error>;

  #[rpc(name = "getnetworkinfo")]
  fn get_network_info(&self) -> Result<GetNetworkInfoResult, jsonrpc_core::Error>;

  #[rpc(name = "getbalances")]
  fn get_balances(&self) -> Result<GetBalancesResult, jsonrpc_core::Error>;

  #[rpc(name = "getblockhash")]
  fn get_block_hash(&self, height: usize) -> Result<BlockHash, jsonrpc_core::Error>;

  #[rpc(name = "getblockheader")]
  fn get_block_header(
    &self,
    block_hash: BlockHash,
    verbose: bool,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "getblockstats")]
  fn get_block_stats(&self, height: usize) -> Result<GetBlockStatsResult, jsonrpc_core::Error>;

  #[rpc(name = "getblock")]
  fn get_block(&self, blockhash: BlockHash, verbosity: u64) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "getblockcount")]
  fn get_block_count(&self) -> Result<u64, jsonrpc_core::Error>;

  #[rpc(name = "gettxout")]
  fn get_tx_out(
    &self,
    txid: Txid,
    vout: u32,
    include_mempool: Option<bool>,
  ) -> Result<Option<GetTxOutResult>, jsonrpc_core::Error>;

  #[rpc(name = "getwalletinfo")]
  fn get_wallet_info(&self) -> Result<GetWalletInfoResult, jsonrpc_core::Error>;

  #[rpc(name = "createrawtransaction")]
  fn create_raw_transaction(
    &self,
    utxos: Vec<CreateRawTransactionInput>,
    outs: HashMap<String, f64>,
    locktime: Option<i64>,
    replaceable: Option<bool>,
  ) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "createwallet")]
  fn create_wallet(
    &self,
    name: String,
    disable_private_keys: Option<bool>,
    blank: Option<bool>,
    passphrase: Option<String>,
    avoid_reuse: Option<bool>,
  ) -> Result<LoadWalletResult, jsonrpc_core::Error>;

  #[rpc(name = "fundrawtransaction")]
  fn fund_raw_transaction(
    &self,
    tx: String,
    options: Option<FundRawTransactionOptions>,
    is_witness: Option<bool>,
  ) -> Result<FundRawTransactionResult, jsonrpc_core::Error>;

  #[rpc(name = "signrawtransactionwithwallet")]
  fn sign_raw_transaction_with_wallet(
    &self,
    tx: String,
    utxos: Option<Vec<SignRawTransactionInput>>,
    sighash_type: Option<()>,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "sendrawtransaction")]
  fn send_raw_transaction(&self, tx: String) -> Result<String, jsonrpc_core::Error>;

  #[rpc(name = "sendtoaddress")]
  fn send_to_address(
    &self,
    address: Address<NetworkUnchecked>,
    amount: f64,
    comment: Option<String>,
    comment_to: Option<String>,
    subtract_fee: Option<bool>,
    replaceable: Option<bool>,
    confirmation_target: Option<u32>,
    estimate_mode: Option<EstimateMode>,
    avoid_reuse: Option<bool>,
    fee_rate: Option<f64>,
    verbose: Option<bool>,
  ) -> Result<Txid, jsonrpc_core::Error>;

  #[rpc(name = "gettransaction")]
  fn get_transaction(
    &self,
    txid: Txid,
    include_watchonly: Option<bool>,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "getrawtransaction")]
  fn get_raw_transaction(
    &self,
    txid: Txid,
    verbose: Option<bool>,
    blockhash: Option<BlockHash>,
  ) -> Result<Value, jsonrpc_core::Error>;

  #[rpc(name = "listunspent")]
  fn list_unspent(
    &self,
    minconf: Option<usize>,
    maxconf: Option<usize>,
    address: Option<Address<NetworkUnchecked>>,
    include_unsafe: Option<bool>,
    query_options: Option<String>,
  ) -> Result<Vec<ListUnspentResultEntry>, jsonrpc_core::Error>;

  #[rpc(name = "listlockunspent")]
  fn list_lock_unspent(&self) -> Result<Vec<JsonOutPoint>, jsonrpc_core::Error>;

  #[rpc(name = "getrawchangeaddress")]
  fn get_raw_change_address(
    &self,
    address_type: Option<bitcoincore_rpc::json::AddressType>,
  ) -> Result<Address, jsonrpc_core::Error>;

  #[rpc(name = "getdescriptorinfo")]
  fn get_descriptor_info(
    &self,
    desc: String,
  ) -> Result<GetDescriptorInfoResult, jsonrpc_core::Error>;

  #[rpc(name = "importdescriptors")]
  fn import_descriptors(
    &self,
    req: Vec<ImportDescriptors>,
  ) -> Result<Vec<ImportMultiResult>, jsonrpc_core::Error>;

  #[rpc(name = "getnewaddress")]
  fn get_new_address(
    &self,
    label: Option<String>,
    address_type: Option<bitcoincore_rpc::json::AddressType>,
  ) -> Result<Address, jsonrpc_core::Error>;

  #[rpc(name = "listtransactions")]
  fn list_transactions(
    &self,
    label: Option<String>,
    count: Option<u16>,
    skip: Option<usize>,
    include_watchonly: Option<bool>,
  ) -> Result<Vec<ListTransactionResult>, jsonrpc_core::Error>;

  #[rpc(name = "lockunspent")]
  fn lock_unspent(
    &self,
    unlock: bool,
    outputs: Vec<JsonOutPoint>,
  ) -> Result<bool, jsonrpc_core::Error>;

  #[rpc(name = "listdescriptors")]
  fn list_descriptors(
    &self,
    _with_private_keys: Option<bool>,
  ) -> Result<ListDescriptorsResult, jsonrpc_core::Error>;

  #[rpc(name = "loadwallet")]
  fn load_wallet(&self, wallet: String) -> Result<LoadWalletResult, jsonrpc_core::Error>;

  #[rpc(name = "listwallets")]
  fn list_wallets(&self) -> Result<Vec<String>, jsonrpc_core::Error>;

  #[rpc(name = "listwalletdir")]
  fn list_wallet_dir(&self) -> Result<ListWalletDirResult, jsonrpc_core::Error>;

  #[rpc(name = "walletprocesspsbt")]
  fn wallet_process_psbt(
    &self,
    psbt: String,
    sign: Option<bool>,
    sighash_type: Option<()>,
    bip32derivs: Option<bool>,
  ) -> Result<WalletProcessPsbtResult, jsonrpc_core::Error>;

  #[rpc(name = "finalizepsbt")]
  fn finalize_psbt(
    &self,
    psbt: String,
    extract: Option<bool>,
  ) -> Result<FinalizePsbtResult, jsonrpc_core::Error>;
}
