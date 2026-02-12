use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct FilteredInscriptionData {
  pub(super) cursed_or_vindicated: bool,
  pub(super) offset: u64,
}

#[derive(Default)]
pub(super) struct InscribedOffset {
  pub(super) count: u32,
  pub(super) filtered_cursed_or_vindicated: Option<bool>,
  pub(super) indexed_inscription_id: Option<InscriptionId>,
}

impl InscribedOffset {
  pub(super) fn insert_filtered(&mut self, cursed_or_vindicated: bool) {
    self.count += 1;

    if self.count == 1 {
      self.filtered_cursed_or_vindicated = Some(cursed_or_vindicated);
    }
  }

  pub(super) fn insert_indexed(&mut self, inscription_id: InscriptionId) {
    self.count += 1;

    if self.count == 1 {
      self.indexed_inscription_id = Some(inscription_id);
    }
  }
}

pub(super) fn parse_filtered_inscription_data(data: &[u8]) -> Vec<FilteredInscriptionData> {
  let mut parsed = Vec::new();
  let mut byte_offset = 0;

  while byte_offset < data.len() {
    let (offset, varint_len) = varint::decode(&data[byte_offset..]).unwrap();
    byte_offset += varint_len;

    let cursed_or_vindicated = data[byte_offset] != 0;
    byte_offset += 1;

    parsed.push(FilteredInscriptionData {
      cursed_or_vindicated,
      offset: u64::try_from(offset).unwrap(),
    });
  }

  parsed
}

pub(super) fn push_filtered_inscription_data(
  data: &mut Vec<u8>,
  offset: u64,
  cursed_or_vindicated: bool,
) {
  varint::encode_to_vec(offset.into(), data);
  data.push(u8::from(cursed_or_vindicated));
}

pub(super) fn record_filtered_inscription_data(
  filtered_inscription_data_cache: &mut Option<&mut HashMap<OutPoint, Vec<u8>>>,
  track_filtered_inscription_data: bool,
  satpoint: SatPoint,
  op_return: bool,
  cursed_or_vindicated: bool,
) {
  if !track_filtered_inscription_data || op_return || Index::is_special_outpoint(satpoint.outpoint)
  {
    return;
  }

  let Some(cache) = filtered_inscription_data_cache.as_deref_mut() else {
    return;
  };

  push_filtered_inscription_data(
    cache.entry(satpoint.outpoint).or_default(),
    satpoint.offset,
    cursed_or_vindicated,
  );
}
