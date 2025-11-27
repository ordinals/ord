use super::*;

pub(crate) trait Entry: Sized {
  type Value;

  fn load(value: Self::Value) -> Self;

  fn store(self) -> Self::Value;
}

pub(super) type HeaderValue = [u8; 80];

impl Entry for Header {
  type Value = HeaderValue;

  fn load(value: Self::Value) -> Self {
    consensus::encode::deserialize(&value).unwrap()
  }

  fn store(self) -> Self::Value {
    let mut buffer = [0; 80];
    let len = self
      .consensus_encode(&mut buffer.as_mut_slice())
      .expect("in-memory writers don't error");
    debug_assert_eq!(len, buffer.len());
    buffer
  }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct InscriptionEntry {
  pub charms: u16,
  pub fee: u64,
  pub height: u32,
  pub id: InscriptionId,
  pub inscription_number: i32,
  pub parents: Vec<u32>,
  pub sat: Option<Sat>,
  pub sequence_number: u32,
  pub timestamp: u32,
}

pub(crate) type InscriptionEntryValue = (
  u16,                // charms
  u64,                // fee
  u32,                // height
  InscriptionIdValue, // inscription id
  i32,                // inscription number
  Vec<u32>,           // parents
  Option<u64>,        // sat
  u32,                // sequence number
  u32,                // timestamp
);

impl Entry for InscriptionEntry {
  type Value = InscriptionEntryValue;

  #[rustfmt::skip]
  fn load(
    (
      charms,
      fee,
      height,
      id,
      inscription_number,
      parents,
      sat,
      sequence_number,
      timestamp,
    ): InscriptionEntryValue,
  ) -> Self {
    Self {
      charms,
      fee,
      height,
      id: InscriptionId::load(id),
      inscription_number,
      parents,
      sat: sat.map(Sat),
      sequence_number,
      timestamp,
    }
  }

  fn store(self) -> Self::Value {
    (
      self.charms,
      self.fee,
      self.height,
      self.id.store(),
      self.inscription_number,
      self.parents,
      self.sat.map(Sat::n),
      self.sequence_number,
      self.timestamp,
    )
  }
}

pub(crate) type InscriptionIdValue = (u128, u128, u32);

impl Entry for InscriptionId {
  type Value = InscriptionIdValue;

  fn load(value: Self::Value) -> Self {
    let (head, tail, index) = value;
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

    Self {
      txid: Txid::from_byte_array(array),
      index,
    }
  }

  fn store(self) -> Self::Value {
    let txid_entry = self.txid.store();
    let little_end = u128::from_le_bytes(txid_entry[..16].try_into().unwrap());
    let big_end = u128::from_le_bytes(txid_entry[16..].try_into().unwrap());
    (little_end, big_end, self.index)
  }
}

pub(super) type OutPointValue = [u8; 36];

impl Entry for OutPoint {
  type Value = OutPointValue;

  fn load(value: Self::Value) -> Self {
    Decodable::consensus_decode(&mut bitcoin::io::Cursor::new(value)).unwrap()
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
    Decodable::consensus_decode(&mut bitcoin::io::Cursor::new(value)).unwrap()
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
    let n = u128::from(base) | (u128::from(delta) << 51);
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
  fn inscription_entry() {
    let id = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefi0"
      .parse::<InscriptionId>()
      .unwrap();

    let entry = InscriptionEntry {
      charms: 0,
      fee: 1,
      height: 2,
      id,
      inscription_number: 3,
      parents: vec![4, 5, 6],
      sat: Some(Sat(7)),
      sequence_number: 8,
      timestamp: 9,
    };

    let value = (0, 1, 2, id.store(), 3, vec![4, 5, 6], Some(7), 8, 9);

    assert_eq!(entry.clone().store(), value);
    assert_eq!(InscriptionEntry::load(value), entry);
  }

  #[test]
  fn inscription_id_entry() {
    let inscription_id = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefi0"
      .parse::<InscriptionId>()
      .unwrap();

    assert_eq!(
      inscription_id.store(),
      (
        0x0123456789abcdef0123456789abcdef,
        0x0123456789abcdef0123456789abcdef,
        0
      )
    );

    assert_eq!(
      InscriptionId::load((
        0x0123456789abcdef0123456789abcdef,
        0x0123456789abcdef0123456789abcdef,
        0
      )),
      inscription_id
    );
  }

  #[test]
  fn parent_entry_index() {
    let inscription_id = "0000000000000000000000000000000000000000000000000000000000000000i1"
      .parse::<InscriptionId>()
      .unwrap();

    assert_eq!(inscription_id.store(), (0, 0, 1));

    assert_eq!(InscriptionId::load((0, 0, 1)), inscription_id);

    let inscription_id = "0000000000000000000000000000000000000000000000000000000000000000i256"
      .parse::<InscriptionId>()
      .unwrap();

    assert_eq!(inscription_id.store(), (0, 0, 256));

    assert_eq!(InscriptionId::load((0, 0, 256)), inscription_id);
  }

  #[test]
  fn header() {
    let expected = [
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
      26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
      49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71,
      72, 73, 74, 75, 76, 77, 78, 79,
    ];

    let header = Header::load(expected);
    let actual = header.store();

    assert_eq!(actual, expected);
  }
}
