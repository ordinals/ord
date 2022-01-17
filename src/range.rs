use super::*;

pub(crate) fn run(height: u64, name_range: bool) -> Result {
  let mut start = 0;

  for i in 0..height {
    if subsidy(i) == 0 {
      break;
    }

    start += subsidy(i);
  }

  if name_range {
    println!("[{},{})", name(start), name(start + subsidy(height)));
  } else {
    println!("[{},{})", start, start + subsidy(height));
  }

  Ok(())
}
