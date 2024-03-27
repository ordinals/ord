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

pub(crate) use transactions::Transactions;

pub use {
  entry::Entry, etching::Etching, file::File, mode::Mode, plan::Plan, range::Range, terms::Terms,
};

pub mod entry;
mod etching;
pub mod file;
pub mod mode;
pub mod plan;
mod range;
mod terms;
mod transactions;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub commit_psbt: Option<String>,
  pub inscriptions: Vec<InscriptionInfo>,
  pub parent: Option<InscriptionId>,
  pub reveal: Txid,
  pub reveal_psbt: Option<String>,
  pub rune: Option<RuneInfo>,
  pub total_fees: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct InscriptionInfo {
  pub destination: Address<NetworkUnchecked>,
  pub id: InscriptionId,
  pub location: SatPoint,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RuneInfo {
  pub destination: Option<Address<NetworkUnchecked>>,
  pub location: Option<OutPoint>,
  pub rune: SpacedRune,
}

#[derive(Clone, Debug)]
pub struct ParentInfo {
  pub destination: Address,
  pub id: InscriptionId,
  pub location: SatPoint,
  pub tx_out: TxOut,
}
