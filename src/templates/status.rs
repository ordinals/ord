use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Indices {
  pub addresses: bool,
  pub inscriptions: bool,
  pub runes: bool,
  pub sats: bool,
  pub transactions: bool,
}

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusHtml {
  pub blessed_inscriptions: u64,
  pub chain: Chain,
  pub cursed_inscriptions: u64,
  pub height: Option<u32>,
  pub indices: Indices,
  pub initial_sync_time: Duration,
  pub inscriptions: u64,
  pub json_api: bool,
  pub lost_sats: u64,
  pub minimum_rune_for_next_block: Rune,
  pub runes: u64,
  pub started: DateTime<Utc>,
  pub unrecoverably_reorged: bool,
  pub uptime: Duration,
}

impl PageContent for StatusHtml {
  fn title(&self) -> String {
    "Status".into()
  }
}
