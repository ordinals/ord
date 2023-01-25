use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub starting_sats: Vec<Sat>,
}

pub(crate) fn run() -> Result {
  let mut starting_sats = Vec::new();
  for sat in Epoch::STARTING_SATS {
    starting_sats.push(sat);
  }

  print_json(Output { starting_sats })?;

  Ok(())
}
