use super::*;

pub(crate) fn run(height: u64) -> Result {
  let mut mined = 0;

  for i in 0..height {
    mined += subsidy(i);
  }

  println!("{} {}", mined, mined + subsidy(height));

  Ok(())
}
