use crate::{InscriptionId, SatPoint};

/// An event from indexing which can be optionally emitted by setting a
/// channel sender using `set_event_sender`.
#[derive(Debug, Clone)]
pub enum Event {
  /// Newly created inscriptions will include additional metadata including
  /// rarity, cursed status, charms, etc.
  InscriptionCreated {
    id: InscriptionId,
    location: Option<SatPoint>,
    sequence_number: u32,
    block_height: u32,
    charms: u16,
    parent_inscription_id: Option<InscriptionId>,
  },
  InscriptionMoved {
    id: InscriptionId,
    old_location: SatPoint,
    new_location: SatPoint,
    sequence_number: u32,
    block_height: u32,
  },
}
