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
    let mut buffer = Cursor::new([0; 80]);
    let len = self
      .consensus_encode(&mut buffer)
      .expect("in-memory writers don't error");
    let buffer = buffer.into_inner();
    debug_assert_eq!(len, buffer.len());
    buffer
  }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct RuneEntry {
  pub burned: u128,
  pub deadline: Option<u32>,
  pub divisibility: u8,
  pub end: Option<u32>,
  pub etching: Txid,
  pub limit: Option<u128>,
  pub mints: u64,
  pub number: u64,
  pub rune: Rune,
  pub spacers: u32,
  pub supply: u128,
  pub symbol: Option<char>,
  pub timestamp: u32,
}

pub(super) type RuneEntryValue = (
  u128,         // burned
  Option<u32>,  // deadline
  u8,           // divisibility
  Option<u32>,  // end
  (u128, u128), // etching
  Option<u128>, // limit
  (
    u64, // mints
    u64, // number
  ),
  u128,         // rune
  u32,          // spacers
  u128,         // supply
  Option<char>, // symbol
  u32,          // timestamp
);

impl RuneEntry {
  pub(crate) fn spaced_rune(&self) -> SpacedRune {
    SpacedRune {
      rune: self.rune,
      spacers: self.spacers,
    }
  }
}

impl Default for RuneEntry {
  fn default() -> Self {
    Self {
      burned: 0,
      deadline: None,
      divisibility: 0,
      end: None,
      etching: Txid::all_zeros(),
      limit: None,
      mints: 0,
      number: 0,
      rune: Rune(0),
      spacers: 0,
      supply: 0,
      symbol: None,
      timestamp: 0,
    }
  }
}

impl Entry for RuneEntry {
  type Value = RuneEntryValue;

  fn load(
    (
      burned,
      deadline,
      divisibility,
      end,
      etching,
      limit,
      (mints, number),
      rune,
      spacers,
      supply,
      symbol,
      timestamp,
    ): RuneEntryValue,
  ) -> Self {
    Self {
      burned,
      deadline,
      divisibility,
      end,
      etching: {
        let low = etching.0.to_le_bytes();
        let high = etching.1.to_le_bytes();
        Txid::from_byte_array([
          low[0], low[1], low[2], low[3], low[4], low[5], low[6], low[7], low[8], low[9], low[10],
          low[11], low[12], low[13], low[14], low[15], high[0], high[1], high[2], high[3], high[4],
          high[5], high[6], high[7], high[8], high[9], high[10], high[11], high[12], high[13],
          high[14], high[15],
        ])
      },
      limit,
      mints,
      number,
      rune: Rune(rune),
      spacers,
      supply,
      symbol,
      timestamp,
    }
  }

  fn store(self) -> Self::Value {
    (
      self.burned,
      self.deadline,
      self.divisibility,
      self.end,
      {
        let bytes = self.etching.to_byte_array();
        (
          u128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
          ]),
          u128::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
            bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
          ]),
        )
      },
      self.limit,
      (self.mints, self.number),
      self.rune.0,
      self.spacers,
      self.supply,
      self.symbol,
      self.timestamp,
    )
  }
}

pub(super) type RuneIdValue = (u32, u16);

impl Entry for RuneId {
  type Value = RuneIdValue;

  fn load((height, index): Self::Value) -> Self {
    Self { height, index }
  }

  fn store(self) -> Self::Value {
    (self.height, self.index)
  }
}

#[derive(Debug)]
pub(crate) struct InscriptionEntry {
  pub(crate) charms: u16,
  pub(crate) fee: u64,
  pub(crate) height: u32,
  pub(crate) id: InscriptionId,
  pub(crate) inscription_number: i32,
  pub(crate) parent: Option<u32>,
  pub(crate) sat: Option<Sat>,
  pub(crate) sequence_number: u32,
  pub(crate) timestamp: u32,
}

pub(crate) type InscriptionEntryValue = (
  u16,                // charms
  u64,                // fee
  u32,                // height
  InscriptionIdValue, // inscription id
  i32,                // inscription number
  Option<u32>,        // parent
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
      parent,
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
      parent,
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
      self.parent,
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
  fn rune_entry() {
    let entry = RuneEntry {
      burned: 1,
      deadline: Some(2),
      divisibility: 3,
      end: Some(4),
      etching: Txid::from_byte_array([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D,
        0x1E, 0x1F,
      ]),
      limit: Some(5),
      mints: 11,
      number: 6,
      rune: Rune(7),
      spacers: 8,
      supply: 9,
      symbol: Some('a'),
      timestamp: 10,
    };

    let value = (
      1,
      Some(2),
      3,
      Some(4),
      (
        0x0F0E0D0C0B0A09080706050403020100,
        0x1F1E1D1C1B1A19181716151413121110,
      ),
      Some(5),
      (11, 6),
      7,
      8,
      9,
      Some('a'),
      10,
    );

    assert_eq!(entry.store(), value);
    assert_eq!(RuneEntry::load(value), entry);
  }

  #[test]
  fn rune_id_entry() {
    assert_eq!(
      RuneId {
        height: 1,
        index: 2,
      }
      .store(),
      (1, 2),
    );

    assert_eq!(
      RuneId {
        height: 1,
        index: 2,
      },
      RuneId::load((1, 2)),
    );
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
