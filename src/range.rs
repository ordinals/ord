use super::*;

pub(crate) fn run(height: u64) -> Result {
  let mut start = 0;

  for i in 0..height {
    if subsidy(i) == 0 {
      break;
    }

    start += subsidy(i);
  }

  println!("[{},{})", start, start + subsidy(height));

  Ok(())
}
