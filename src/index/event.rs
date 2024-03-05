use bitcoin::{OutPoint, Txid};

use crate::{InscriptionId, RuneId, SatPoint};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
  InscriptionCreated {
    block_height: u32,
    charms: u16,
    inscription_id: InscriptionId,
    location: Option<SatPoint>,
    parent_inscription_id: Option<InscriptionId>,
    sequence_number: u32,
  },
  InscriptionTransferred {
    block_height: u32,
    inscription_id: InscriptionId,
    new_location: SatPoint,
    old_location: SatPoint,
    sequence_number: u32,
  },
  RuneEtched {
    block_height: u32,
    txid: Txid,
    rune_id: RuneId,
  },
  RuneClaimed {
    block_height: u32,
    txid: Txid,
    rune_id: RuneId,
    amount: u128,
  },
  RuneTransferred {
    block_height: u32,
    txid: Txid,
    rune_id: RuneId,
    amount: u128,
    outpoint: OutPoint,
  },
  RuneBurned {
    block_height: u32,
    txid: Txid,
    rune_id: RuneId,
    amount: u128,
  },
}
