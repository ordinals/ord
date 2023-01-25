use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub sat: Sat,
}

pub(crate) fn run() -> Result {
  let mut output = Vec::new();
  for sat in Epoch::STARTING_SATS {
    output.push(sat);
  }

  print_json(output)?;

  Ok(())
}
