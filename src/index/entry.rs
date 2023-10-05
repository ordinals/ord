use super::*;

pub(super) trait Entry: Sized {
  type Value;

  fn load(value: Self::Value) -> Self;

  fn store(self) -> Self::Value;
}

pub(super) type BlockHashValue = [u8; 32];

impl Entry for BlockHash {
  type Value = BlockHashValue;

  fn load(value: Self::Value) -> Self {
    BlockHash::from_raw_hash(Hash::from_byte_array(value))
  }

  fn store(self) -> Self::Value {
    *self.as_ref()
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct RuneEntry {
  pub(crate) divisibility: u128,
  pub(crate) rarity: Rarity,
  pub(crate) rune: Rune,
  pub(crate) supply: u128,
}

pub(super) type RuneEntryValue = (u128, u128, u128, u8);

impl Entry for RuneEntry {
  type Value = RuneEntryValue;

  fn load((divisibility, rune, supply, rarity): RuneEntryValue) -> Self {
    Self {
      divisibility,
      rarity: Rarity::try_from(rarity).unwrap(),
      rune: Rune(rune),
      supply,
    }
  }

  fn store(self) -> Self::Value {
    (
      self.divisibility,
      self.rune.0,
      self.supply,
      self.rarity.into(),
    )
  }
}

impl Entry for RuneId {
  type Value = u64;

  fn load(value: u64) -> Self {
    let bytes = value.to_le_bytes();
    Self {
      height: u64::from_le_bytes([
        bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], 0, 0,
      ]),
      index: u16::from_le_bytes([bytes[0], bytes[1]]),
    }
  }

  fn store(self) -> Self::Value {
    let height = self.height.to_le_bytes();
    let index = self.index.to_le_bytes();

    u64::from_le_bytes([
      index[0], index[1], height[0], height[1], height[2], height[3], height[4], height[5],
    ])
  }
}

#[derive(Debug)]
pub(crate) struct InscriptionEntry {
  pub(crate) fee: u64,
  pub(crate) height: u64,
  pub(crate) inscription_number: i64,
  pub(crate) sequence_number: u64,
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) sat: Option<Sat>,
  pub(crate) timestamp: u32,
}

pub(crate) type InscriptionEntryValue = (u64, u64, i64, u64, ParentValue, u64, u32);

impl Entry for InscriptionEntry {
  type Value = InscriptionEntryValue;

  fn load(
    (fee, height, inscription_number, sequence_number, parent, sat, timestamp): InscriptionEntryValue,
  ) -> Self {
    Self {
      fee,
      height,
      inscription_number,
      sequence_number,
      parent: ParentEntry::load(parent),
      sat: if sat == u64::MAX {
        None
      } else {
        Some(Sat(sat))
      },
      timestamp,
    }
  }

  fn store(self) -> Self::Value {
    (
      self.fee,
      self.height,
      self.inscription_number,
      self.sequence_number,
      self.parent.store(),
      match self.sat {
        Some(sat) => sat.n(),
        None => u64::MAX,
      },
      self.timestamp,
    )
  }
}

pub(super) type InscriptionIdValue = [u8; 36];

impl Entry for InscriptionId {
  type Value = InscriptionIdValue;

  fn load(value: Self::Value) -> Self {
    let (txid, index) = value.split_at(32);
    Self {
      txid: Txid::from_raw_hash(Hash::from_slice(txid).unwrap()),
      index: u32::from_be_bytes(index.try_into().unwrap()),
    }
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(self.txid.as_ref());
    index.copy_from_slice(&self.index.to_be_bytes());
    value
  }
}

type ParentValue = (u128, u128, u32);
type ParentEntry = Option<InscriptionId>;

impl Entry for ParentEntry {
  type Value = ParentValue;

  fn load(value: Self::Value) -> Self {
    if (0, 0, 0) == value {
      None
    } else {
      let (head, tail, index) = value;
      let head_array = head.to_le_bytes();
      let tail_array = tail.to_le_bytes();
      let index_array = index.to_be_bytes();
      let array = [
        head_array[0],
        head_array[1],
        head_array[2],
        head_array[3],
        head_array[4],
        head_array[5],
        head_array[6],
        head_array[7],
        head_array[8],
        head_array[9],
        head_array[10],
        head_array[11],
        head_array[12],
        head_array[13],
        head_array[14],
        head_array[15],
        tail_array[0],
        tail_array[1],
        tail_array[2],
        tail_array[3],
        tail_array[4],
        tail_array[5],
        tail_array[6],
        tail_array[7],
        tail_array[8],
        tail_array[9],
        tail_array[10],
        tail_array[11],
        tail_array[12],
        tail_array[13],
        tail_array[14],
        tail_array[15],
        index_array[0],
        index_array[1],
        index_array[2],
        index_array[3],
      ];

      Some(InscriptionId::load(array))
    }
  }

  fn store(self) -> Self::Value {
    if let Some(inscription_id) = self {
      let txid_entry = inscription_id.txid.store();
      let little_end = u128::from_le_bytes(txid_entry[..16].try_into().unwrap());
      let big_end = u128::from_le_bytes(txid_entry[16..].try_into().unwrap());
      (little_end, big_end, inscription_id.index)
    } else {
      (0, 0, 0)
    }
  }
}

pub(super) type OutPointValue = [u8; 36];

impl Entry for OutPoint {
  type Value = OutPointValue;

  fn load(value: Self::Value) -> Self {
    Decodable::consensus_decode(&mut io::Cursor::new(value)).unwrap()
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 36];
    self.consensus_encode(&mut value.as_mut_slice()).unwrap();
    value
  }
}

pub(super) type SatPointValue = [u8; 44];

impl Entry for SatPoint {
  type Value = SatPointValue;

  fn load(value: Self::Value) -> Self {
    Decodable::consensus_decode(&mut io::Cursor::new(value)).unwrap()
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 44];
    self.consensus_encode(&mut value.as_mut_slice()).unwrap();
    value
  }
}

pub(super) type SatRange = (u64, u64);

impl Entry for SatRange {
  type Value = [u8; 11];

  fn load([b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10]: Self::Value) -> Self {
    let raw_base = u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, 0]);

    // 51 bit base
    let base = raw_base & ((1 << 51) - 1);

    let raw_delta = u64::from_le_bytes([b6, b7, b8, b9, b10, 0, 0, 0]);

    // 33 bit delta
    let delta = raw_delta >> 3;

    (base, base + delta)
  }

  fn store(self) -> Self::Value {
    let base = self.0;
    let delta = self.1 - self.0;
    let n = u128::from(base) | u128::from(delta) << 51;
    n.to_le_bytes()[0..11].try_into().unwrap()
  }
}

pub(super) type TxidValue = [u8; 32];

impl Entry for Txid {
  type Value = TxidValue;

  fn load(value: Self::Value) -> Self {
    Txid::from_byte_array(value)
  }

  fn store(self) -> Self::Value {
    Txid::to_byte_array(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parent_entry() {
    let inscription_id: Option<InscriptionId> = None;

    assert_eq!(inscription_id.store(), (0, 0, 0));
    assert_eq!(
      <Option<InscriptionId> as Entry>::load((0, 0, 0)),
      inscription_id
    );

    let inscription_id = Some(
      "0000000000000000000000000000000000000000000000000000000000000000i1"
        .parse::<InscriptionId>()
        .unwrap(),
    );

    assert_eq!(inscription_id.store(), (0, 0, 1));
    assert_eq!(
      <Option<InscriptionId> as Entry>::load((0, 0, 1)),
      inscription_id
    );

    let inscription_id = Some(
      "ffffffffffffffffffffffffffffffff00000000000000000000000000000000i0"
        .parse::<InscriptionId>()
        .unwrap(),
    );

    assert_eq!(inscription_id.store(), (0, u128::MAX, 0));
    assert_eq!(
      <Option<InscriptionId> as Entry>::load((0, u128::MAX, 0)),
      inscription_id
    );
  }

  #[test]
  fn parent_entry_individual_byte_order() {
    let inscription_id = Some(
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefi0"
        .parse::<InscriptionId>()
        .unwrap(),
    );

    assert_eq!(
      inscription_id.store(),
      (
        0x0123456789abcdef0123456789abcdef,
        0x0123456789abcdef0123456789abcdef,
        0
      )
    );

    assert_eq!(
      <Option<InscriptionId> as Entry>::load((
        0x0123456789abcdef0123456789abcdef,
        0x0123456789abcdef0123456789abcdef,
        0
      )),
      inscription_id
    );
  }

  #[test]
  fn parent_entry_index() {
    let inscription_id = Some(
      "0000000000000000000000000000000000000000000000000000000000000000i1"
        .parse::<InscriptionId>()
        .unwrap(),
    );

    assert_eq!(inscription_id.store(), (0, 0, 1));

    assert_eq!(
      <Option<InscriptionId> as Entry>::load((0, 0, 1)),
      inscription_id
    );
  }

  #[test]
  fn rune_entry() {
    let rune_entry = RuneEntry {
      divisibility: 1,
      rarity: Rarity::Rare,
      rune: Rune(3),
      supply: 4,
    };

    assert_eq!(rune_entry.store(), (1, 3, 4, 2));

    assert_eq!(RuneEntry::load((1, 3, 4, 2)), rune_entry);
  }
}
