use {
  super::{
    target_as_block_hash, BlockHash, Chain, Deserialize, Height, InscriptionId, OutPoint, Pile,
    Rarity, SatPoint, Serialize, SpacedRune, TxMerkleNode, TxOut,
  },
  serde_hex::{SerHex, Strict},
};

pub use crate::templates::{
  BlocksHtml as Blocks, RuneHtml as Rune, RunesHtml as Runes, StatusHtml as Status,
  TransactionHtml as Transaction,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
  pub hash: BlockHash,
  pub target: BlockHash,
  pub best_height: u32,
  pub height: u32,
  pub inscriptions: Vec<InscriptionId>,
}

impl Block {
  pub(crate) fn new(
    block: bitcoin::Block,
    height: Height,
    best_height: Height,
    inscriptions: Vec<InscriptionId>,
  ) -> Self {
    Self {
      hash: block.header.block_hash(),
      target: target_as_block_hash(block.header.target()),
      height: height.0,
      best_height: best_height.0,
      inscriptions,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockInfo {
  pub average_fee: u64,
  pub average_fee_rate: u64,
  pub bits: u32,
  #[serde(with = "SerHex::<Strict>")]
  pub chainwork: [u8; 32],
  pub confirmations: i32,
  pub difficulty: f64,
  pub hash: BlockHash,
  pub height: u32,
  pub max_fee: u64,
  pub max_fee_rate: u64,
  pub max_tx_size: u32,
  pub median_fee: u64,
  pub median_time: Option<u64>,
  pub merkle_root: TxMerkleNode,
  pub min_fee: u64,
  pub min_fee_rate: u64,
  pub next_block: Option<BlockHash>,
  pub nonce: u32,
  pub previous_block: Option<BlockHash>,
  pub subsidy: u64,
  pub target: BlockHash,
  pub timestamp: u64,
  pub total_fee: u64,
  pub total_size: usize,
  pub total_weight: usize,
  pub transaction_count: u64,
  pub version: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Children {
  pub ids: Vec<InscriptionId>,
  pub more: bool,
  pub page: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Inscription {
  pub address: Option<String>,
  pub charms: Vec<String>,
  pub children: Vec<InscriptionId>,
  pub content_length: Option<usize>,
  pub content_type: Option<String>,
  pub fee: u64,
  pub height: u32,
  pub id: InscriptionId,
  pub next: Option<InscriptionId>,
  pub number: i32,
  pub parent: Option<InscriptionId>,
  pub previous: Option<InscriptionId>,
  pub rune: Option<SpacedRune>,
  pub sat: Option<ordinals::Sat>,
  pub satpoint: SatPoint,
  pub timestamp: i64,
  pub value: Option<u64>,

  // ---- Ordzaar ----
  pub inscription_sequence: u32,
  // ---- Ordzaar ----
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionRecursive {
  pub charms: Vec<String>,
  pub content_type: Option<String>,
  pub content_length: Option<usize>,
  pub fee: u64,
  pub height: u32,
  pub id: InscriptionId,
  pub number: i32,
  pub output: OutPoint,
  pub sat: Option<ordinals::Sat>,
  pub satpoint: SatPoint,
  pub timestamp: i64,
  pub value: Option<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Inscriptions {
  pub ids: Vec<InscriptionId>,
  pub more: bool,
  pub page_index: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub address: Option<String>,
  pub indexed: bool,
  pub inscriptions: Vec<InscriptionId>,
  pub runes: Vec<(SpacedRune, Pile)>,
  pub sat_ranges: Option<Vec<(u64, u64)>>,
  pub script_pubkey: String,
  pub spent: bool,
  pub transaction: String,
  pub value: u64,
}

impl Output {
  pub fn new(
    chain: Chain,
    inscriptions: Vec<InscriptionId>,
    outpoint: OutPoint,
    output: TxOut,
    indexed: bool,
    runes: Vec<(SpacedRune, Pile)>,
    sat_ranges: Option<Vec<(u64, u64)>>,
    spent: bool,
  ) -> Self {
    Self {
      address: chain
        .address_from_script(&output.script_pubkey)
        .ok()
        .map(|address| address.to_string()),
      indexed,
      inscriptions,
      runes,
      sat_ranges,
      script_pubkey: output.script_pubkey.to_asm_string(),
      spent,
      transaction: outpoint.txid.to_string(),
      value: output.value,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Sat {
  pub number: u64,
  pub decimal: String,
  pub degree: String,
  pub name: String,
  pub block: u32,
  pub cycle: u32,
  pub epoch: u32,
  pub period: u32,
  pub offset: u64,
  pub rarity: Rarity,
  pub percentile: String,
  pub satpoint: Option<SatPoint>,
  pub timestamp: i64,
  pub inscriptions: Vec<InscriptionId>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SatInscription {
  pub id: Option<InscriptionId>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SatInscriptions {
  pub ids: Vec<InscriptionId>,
  pub more: bool,
  pub page: u64,
}
