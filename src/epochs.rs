use super::*;

pub(crate) fn run() -> Result {
  for ordinal in EPOCH_ORDINALS {
    println!("{}", ordinal);
  }

  Ok(())
}
