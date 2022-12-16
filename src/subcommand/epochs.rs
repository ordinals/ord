use super::*;

pub(crate) fn run() -> Result {
  for sat in Epoch::STARTING_SATS {
    println!("{}", sat);
  }
  Ok(())
}
