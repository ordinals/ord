use super::*;

pub(crate) trait Entry: Sized {
  type Value;

  fn load(value: Self::Value) -> Self;

  fn store(self) -> Self::Value;
}

pub(super) type BlockHashValue = [u8; 32];

impl Entry for BlockHash {
  type Value = BlockHashValue;

  fn load(value: Self::Value) -> Self {
    BlockHash::from_inner(value)
  }

  fn store(self) -> Self::Value {
    self.into_inner()
  }
}

pub(crate) struct InscriptionEntry {
  pub(crate) fee: u64,
  pub(crate) height: u64,
  pub(crate) number: u64,
  // pub(crate) parent: Option<InsciptionId>,
  pub(crate) sat: Option<Sat>,
  pub(crate) timestamp: u32,
}

// pub(crate) type InscriptionEntryValue = (u64, u64, u64, (u128, u128), u64, u32);
pub(crate) type InscriptionEntryValue = (u64, u64, u64, u64, u32);

impl Entry for InscriptionEntry {
  type Value = InscriptionEntryValue;

  fn load((fee, height, number, sat, timestamp): InscriptionEntryValue) -> Self {
    Self {
      fee,
      height,
      number,
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
      self.number,
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
      txid: Txid::from_inner(txid.try_into().unwrap()),
      index: u32::from_be_bytes(index.try_into().unwrap()),
    }
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(self.txid.as_inner());
    index.copy_from_slice(&self.index.to_be_bytes());
    value
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
    Txid::from_inner(value)
  }

  fn store(self) -> Self::Value {
    Txid::into_inner(self)
  }
}

impl Entry for Option<InscriptionId> {
  type Value = (u128, u128, u32);

  fn load(value: Self::Value) -> Self {
    if (0, 0, u32::MAX) == value {
      None
    } else {
      let (head, tail, index) = value;
      debug_assert_eq!(index, 0);
      let head_array = head.to_le_bytes();
      let tail_array = tail.to_le_bytes();
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
      ];
      let txid = Txid::load(array);
      // TODO: do we want to handle inscriptions not at index 0
      Some(InscriptionId::from(txid))
    }
  }

  fn store(self) -> Self::Value {
    if let Some(inscription_id) = self {
      let txid_entry = inscription_id.txid.store();
      let head = u128::from_le_bytes([
        txid_entry[0],
        txid_entry[1],
        txid_entry[2],
        txid_entry[3],
        txid_entry[4],
        txid_entry[5],
        txid_entry[6],
        txid_entry[7],
        txid_entry[8],
        txid_entry[9],
        txid_entry[10],
        txid_entry[11],
        txid_entry[12],
        txid_entry[13],
        txid_entry[14],
        txid_entry[15],
      ]);

      let tail = u128::from_le_bytes([
        txid_entry[16 + 0],
        txid_entry[16 + 1],
        txid_entry[16 + 2],
        txid_entry[16 + 3],
        txid_entry[16 + 4],
        txid_entry[16 + 5],
        txid_entry[16 + 6],
        txid_entry[16 + 7],
        txid_entry[16 + 8],
        txid_entry[16 + 9],
        txid_entry[16 + 10],
        txid_entry[16 + 11],
        txid_entry[16 + 12],
        txid_entry[16 + 13],
        txid_entry[16 + 14],
        txid_entry[16 + 15],
      ]);

      (head, tail, inscription_id.index)
    } else {
      (0, 0, u32::MAX)
    }
  }
}
