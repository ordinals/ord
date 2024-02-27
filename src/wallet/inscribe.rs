use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    key::PrivateKey,
    key::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
    policy::MAX_STANDARD_TX_WEIGHT,
    secp256k1::{self, constants::SCHNORR_SIGNATURE_SIZE, rand, Secp256k1, XOnlyPublicKey},
    sighash::{Prevouts, SighashCache, TapSighashType},
    taproot::Signature,
    taproot::{ControlBlock, LeafVersion, TapLeafHash, TaprootBuilder},
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, SignRawTransactionInput, Timestamp},
  wallet::transaction_builder::Target,
};

pub use {batch::Batch, batch_entry::BatchEntry, batch_file::Batchfile, mode::Mode};

pub mod batch;
pub mod batch_entry;
pub mod batch_file;
pub mod mode;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct InscriptionInfo {
  pub id: InscriptionId,
  pub location: SatPoint,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub commit_psbt: Option<String>,
  pub inscriptions: Vec<InscriptionInfo>,
  pub parent: Option<InscriptionId>,
  pub reveal: Txid,
  pub reveal_psbt: Option<String>,
  pub total_fees: u64,
}

#[derive(Clone, Debug)]
pub struct ParentInfo {
  pub destination: Address,
  pub id: InscriptionId,
  pub location: SatPoint,
  pub tx_out: TxOut,
}
