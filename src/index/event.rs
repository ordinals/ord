use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
  InscriptionCreated {
    block_height: u32,
    charms: u16,
    inscription_id: InscriptionId,
    location: Option<SatPoint>,
    parent_inscription_ids: Vec<InscriptionId>,
    sequence_number: u32,
    content_hash: String,
  },
  InscriptionTransferred {
    block_height: u32,
    inscription_id: InscriptionId,
    new_location: SatPoint,
    old_location: SatPoint,
    sequence_number: u32,
  },
  BlockStart {
    block_height: u32,
  },
  BlockEnd {
    block_height: u32,
  },
  RuneBurned {
    amount: u128,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
  },
  RuneEtched {
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
  },
  RuneMinted {
    amount: u128,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
  },
  RuneTransferred {
    amount: u128,
    block_height: u32,
    outpoint: OutPoint,
    rune_id: RuneId,
    txid: Txid,
  },
}

pub trait EventHash {
  fn hash(&self) -> String;
}

fn loc_to_bytes(loc: &SatPoint) -> Vec<u8> {
  let mut bytes = Vec::new();

  bytes.extend(loc.outpoint.txid.to_byte_array());

  bytes.extend(loc.outpoint.vout.to_be_bytes());

  bytes.extend(loc.offset.to_be_bytes());

  bytes
}

impl EventHash for Event {
  fn hash(&self) -> String {
    match self {
      Event::InscriptionCreated {
        block_height,
        charms,
        inscription_id,
        location,
        parent_inscription_ids,
        sequence_number,
        content_hash,
      } => {
        let mut bytes = Vec::new();

        bytes.extend(block_height.to_be_bytes());

        bytes.extend(charms.to_be_bytes());

        bytes.extend(inscription_id.value());

        if let Some(loc) = location {
          bytes.extend(loc_to_bytes(loc));
        }

        for id in parent_inscription_ids.iter() {
          bytes.extend(id.value());
        }

        bytes.extend(sequence_number.to_be_bytes());
        bytes.extend(content_hash.as_bytes());

        let digest = bitcoin::hashes::sha256::Hash::hash(&bytes);
        return hex::encode(&digest[0..32]);
      }

      Event::InscriptionTransferred {
        block_height,
        inscription_id,
        new_location,
        old_location,
        sequence_number,
      } => {
        let mut bytes = Vec::new();
        bytes.extend(block_height.to_be_bytes());
        bytes.extend(inscription_id.value());
        bytes.extend(loc_to_bytes(new_location));
        bytes.extend(loc_to_bytes(old_location));
        bytes.extend(sequence_number.to_be_bytes());

        let digest = bitcoin::hashes::sha256::Hash::hash(&bytes);
        return hex::encode(&digest[0..32]);
      }
      _ => {}
    }
    "not impl".to_string()
  }
}
