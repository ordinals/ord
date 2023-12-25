use super::*;

#[derive(Boilerplate)]
pub(crate) struct StatusHtml {
  pub(crate) blessed_inscriptions: u64,
  pub(crate) cursed_inscriptions: u64,
  pub(crate) height: Option<u32>,
  pub(crate) inscriptions: u64,
  pub(crate) lost_sats: u64,
  pub(crate) minimum_rune_for_next_block: Rune,
  pub(crate) rune_index: bool,
  pub(crate) runes: u64,
  pub(crate) sat_index: bool,
  pub(crate) started: DateTime<Utc>,
  pub(crate) transaction_index: bool,
  pub(crate) unrecoverably_reorged: bool,
  pub(crate) uptime: Duration,
}

impl PageContent for StatusHtml {
  fn title(&self) -> String {
    "Status".into()
  }
}
