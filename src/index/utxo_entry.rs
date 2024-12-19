use {
  super::{
    entry::{Entry, SatRange},
    Index,
  },
  ordinals::varint,
  redb::TypeName,
  ref_cast::RefCast,
  std::ops::Deref,
};

enum Sats<'a> {
  Ranges(&'a [u8]),
  Value(u64),
}

/// A `UtxoEntry` stores the following information about an unspent transaction
/// output, depending on the indexing options:
///
/// If `--index-sats`, the full list of sat ranges, stored as a varint followed
/// by that many 11-byte sat range entries, otherwise the total output value
/// stored as a varint.
///
/// If `--index-addresses`, the script pubkey stored as a varint followed by
/// that many bytes of data.
///
/// If `--index-inscriptions`, the list of inscriptions stored as
/// `(sequence_number, offset)`, with the sequence number stored as a u32 and
/// the offset as a varint.
///
/// Note that the list of inscriptions doesn't need an explicit length, it
/// continues until the end of the array.
///
/// A `UtxoEntry` is the read-only value stored in redb as a byte string. A
/// `UtxoEntryBuf` is the writeable version, used for constructing new
/// `UtxoEntry`s. A `ParsedUtxoEntry` is the parsed value.
#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct UtxoEntry {
  bytes: [u8],
}

impl UtxoEntry {
  pub fn parse(&self, index: &Index) -> ParsedUtxoEntry {
    let sats;
    let mut script_pubkey = None;
    let mut inscriptions = None;

    let mut offset = 0;
    if index.index_sats {
      let (num_sat_ranges, varint_len) = varint::decode(&self.bytes).unwrap();
      offset += varint_len;

      let num_sat_ranges: usize = num_sat_ranges.try_into().unwrap();
      let sat_ranges_len = num_sat_ranges * 11;
      sats = Sats::Ranges(&self.bytes[offset..offset + sat_ranges_len]);
      offset += sat_ranges_len;
    } else {
      let (value, varint_len) = varint::decode(&self.bytes).unwrap();
      sats = Sats::Value(value.try_into().unwrap());
      offset += varint_len;
    };

    if index.index_addresses {
      let (script_pubkey_len, varint_len) = varint::decode(&self.bytes[offset..]).unwrap();
      offset += varint_len;

      let script_pubkey_len: usize = script_pubkey_len.try_into().unwrap();
      script_pubkey = Some(&self.bytes[offset..offset + script_pubkey_len]);
      offset += script_pubkey_len;
    }

    if index.index_inscriptions {
      inscriptions = Some(&self.bytes[offset..self.bytes.len()]);
    }

    ParsedUtxoEntry {
      sats,
      script_pubkey,
      inscriptions,
    }
  }

  pub fn to_buf(&self) -> UtxoEntryBuf {
    UtxoEntryBuf {
      vec: self.bytes.to_vec(),
      #[cfg(debug_assertions)]
      state: State::Valid,
    }
  }
}

impl redb::Value for &UtxoEntry {
  type SelfType<'a>
    = &'a UtxoEntry
  where
    Self: 'a;

  type AsBytes<'a>
    = &'a [u8]
  where
    Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    UtxoEntry::ref_cast(data)
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    &value.bytes
  }

  fn type_name() -> TypeName {
    TypeName::new("ord::UtxoEntry")
  }
}

pub struct ParsedUtxoEntry<'a> {
  sats: Sats<'a>,
  script_pubkey: Option<&'a [u8]>,
  inscriptions: Option<&'a [u8]>,
}

impl<'a> ParsedUtxoEntry<'a> {
  pub fn total_value(&self) -> u64 {
    match self.sats {
      Sats::Value(value) => value,
      Sats::Ranges(ranges) => {
        let mut value = 0;
        for chunk in ranges.chunks_exact(11) {
          let range = SatRange::load(chunk.try_into().unwrap());
          value += range.1 - range.0;
        }
        value
      }
    }
  }

  pub fn sat_ranges(&self) -> &'a [u8] {
    let Sats::Ranges(ranges) = self.sats else {
      panic!("sat ranges are missing");
    };
    ranges
  }

  pub fn script_pubkey(&self) -> &'a [u8] {
    self.script_pubkey.unwrap()
  }

  pub fn inscriptions(&self) -> &'a [u8] {
    self.inscriptions.unwrap()
  }

  pub fn parse_inscriptions(&self) -> Vec<(u32, u64)> {
    let inscriptions = self.inscriptions.unwrap();
    let mut byte_offset = 0;
    let mut parsed_inscriptions = Vec::new();

    while byte_offset < inscriptions.len() {
      let sequence_number = u32::from_le_bytes(
        inscriptions[byte_offset..byte_offset + 4]
          .try_into()
          .unwrap(),
      );
      byte_offset += 4;

      let (satpoint_offset, varint_len) = varint::decode(&inscriptions[byte_offset..]).unwrap();
      let satpoint_offset = u64::try_from(satpoint_offset).unwrap();
      byte_offset += varint_len;

      parsed_inscriptions.push((sequence_number, satpoint_offset));
    }

    parsed_inscriptions
  }
}

#[cfg(debug_assertions)]
#[derive(Debug, Eq, PartialEq)]
enum State {
  NeedSats,
  NeedScriptPubkey,
  Valid,
}

#[derive(Debug)]
pub struct UtxoEntryBuf {
  vec: Vec<u8>,
  #[cfg(debug_assertions)]
  state: State,
}

impl UtxoEntryBuf {
  pub fn new() -> Self {
    Self {
      vec: Vec::new(),
      #[cfg(debug_assertions)]
      state: State::NeedSats,
    }
  }

  pub fn push_value(&mut self, value: u64, index: &Index) {
    assert!(!index.index_sats);
    varint::encode_to_vec(value.into(), &mut self.vec);

    #[cfg(debug_assertions)]
    self.advance_state(State::NeedSats, State::NeedScriptPubkey, index);
  }

  pub fn push_sat_ranges(&mut self, sat_ranges: &[u8], index: &Index) {
    assert!(index.index_sats);
    let num_sat_ranges = sat_ranges.len() / 11;
    assert!(num_sat_ranges * 11 == sat_ranges.len());
    varint::encode_to_vec(num_sat_ranges.try_into().unwrap(), &mut self.vec);
    self.vec.extend(sat_ranges);

    #[cfg(debug_assertions)]
    self.advance_state(State::NeedSats, State::NeedScriptPubkey, index);
  }

  pub fn push_script_pubkey(&mut self, script_pubkey: &[u8], index: &Index) {
    assert!(index.index_addresses);
    varint::encode_to_vec(script_pubkey.len().try_into().unwrap(), &mut self.vec);
    self.vec.extend(script_pubkey);

    #[cfg(debug_assertions)]
    self.advance_state(State::NeedScriptPubkey, State::Valid, index);
  }

  pub fn push_inscriptions(&mut self, inscriptions: &[u8], index: &Index) {
    assert!(index.index_inscriptions);
    self.vec.extend(inscriptions);

    #[cfg(debug_assertions)]
    self.advance_state(State::Valid, State::Valid, index);
  }

  pub fn push_inscription(&mut self, sequence_number: u32, satpoint_offset: u64, index: &Index) {
    assert!(index.index_inscriptions);
    self.vec.extend(sequence_number.to_le_bytes());
    varint::encode_to_vec(satpoint_offset.into(), &mut self.vec);

    #[cfg(debug_assertions)]
    self.advance_state(State::Valid, State::Valid, index);
  }

  #[cfg(debug_assertions)]
  fn advance_state(&mut self, expected_state: State, new_state: State, index: &Index) {
    assert!(self.state == expected_state);
    self.state = new_state;

    if self.state == State::NeedScriptPubkey && !index.index_addresses {
      self.state = State::Valid;
    }
  }

  pub fn merged(a: &UtxoEntry, b: &UtxoEntry, index: &Index) -> Self {
    let a_parsed = a.parse(index);
    let b_parsed = b.parse(index);
    let mut merged = Self::new();

    if index.index_sats {
      let sat_ranges = [a_parsed.sat_ranges(), b_parsed.sat_ranges()].concat();
      merged.push_sat_ranges(&sat_ranges, index);
    } else {
      assert!(a_parsed.total_value() == 0);
      assert!(b_parsed.total_value() == 0);
      merged.push_value(0, index);
    }

    if index.index_addresses {
      assert!(a_parsed.script_pubkey().is_empty());
      assert!(b_parsed.script_pubkey().is_empty());
      merged.push_script_pubkey(&[], index);
    }

    if index.index_inscriptions {
      merged.push_inscriptions(a_parsed.inscriptions(), index);
      merged.push_inscriptions(b_parsed.inscriptions(), index);
    }

    merged
  }

  pub fn empty(index: &Index) -> Self {
    let mut utxo_entry = Self::new();

    if index.index_sats {
      utxo_entry.push_sat_ranges(&[], index);
    } else {
      utxo_entry.push_value(0, index);
    }

    if index.index_addresses {
      utxo_entry.push_script_pubkey(&[], index);
    }

    utxo_entry
  }

  pub fn as_ref(&self) -> &UtxoEntry {
    #[cfg(debug_assertions)]
    assert!(self.state == State::Valid);
    UtxoEntry::ref_cast(&self.vec)
  }
}

impl Default for UtxoEntryBuf {
  fn default() -> Self {
    Self::new()
  }
}

impl Deref for UtxoEntryBuf {
  type Target = UtxoEntry;

  fn deref(&self) -> &UtxoEntry {
    self.as_ref()
  }
}
