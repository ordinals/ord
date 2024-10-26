use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusHtml {
  pub address_index: bool,
  pub blessed_inscriptions: u64,
  pub chain: Chain,
  pub cursed_inscriptions: u64,
  pub height: Option<u32>,
  pub initial_sync_time: Duration,
  pub inscription_index: bool,
  pub inscriptions: u64,
  pub json_api: bool,
  pub lost_sats: u64,
  pub minimum_rune_for_next_block: Rune,
  pub rune_index: bool,
  pub runes: u64,
  pub sat_index: bool,
  pub started: DateTime<Utc>,
  pub transaction_index: bool,
  pub unrecoverably_reorged: bool,
  pub uptime: Duration,
}

impl PageContent for StatusHtml {
  fn title(&self) -> String {
    "Status".into()
  }
}
