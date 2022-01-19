use super::*;

pub(crate) fn run() -> Result {
  for ordinal in Epoch::STARTING_ORDINALS {
    println!("{}", ordinal);
  }

  Ok(())
}
