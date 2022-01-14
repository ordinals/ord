use super::*;

pub(crate) fn run(height: u64) -> Result {
  let mut start = SUPPLY as i64 - 1;

  for i in 0..height {
    if subsidy(i) == 0 {
      break;
    }

    start -= subsidy(i) as i64;
  }

  println!("{} {}", start, start as i64 - subsidy(height) as i64);

  Ok(())
}
