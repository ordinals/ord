use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionData {
  pub inscription_id: InscriptionId,
  pub number: i32,
  pub sequence: u32,
  pub genesis_height: u32,
  pub genesis_fee: u64,
  pub sat: Option<Sat>,
  pub satpoint: SatPoint,
  pub timestamp: i64,
}

impl InscriptionData {
  pub fn new(
    genesis_fee: u64,
    genesis_height: u32,
    inscription_id: InscriptionId,
    number: i32,
    sequence: u32,
    sat: Option<Sat>,
    satpoint: SatPoint,
    timestamp: DateTime<Utc>,
  ) -> Self {
    Self {
      inscription_id,
      number,
      sequence,
      genesis_height,
      genesis_fee,
      sat,
      satpoint,
      timestamp: timestamp.timestamp(),
    }
  }
}

#[derive(Deserialize)]
pub struct InscriptionIds {
  pub ids: Vec<InscriptionId>,
}
