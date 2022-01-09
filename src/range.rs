use super::*;

pub(crate) fn run(height: u64) -> Result {
  let mut mined = 0;

  for i in 0..height {
    mined += subsidy(i);
  }

  println!("{} {}", mined, mined + subsidy(height));

  Ok(())
}

fn subsidy(height: u64) -> u64 {
  let subsidy = 50 * COIN_VALUE;

  let halvings = height / 210000;

  if halvings < 64 {
    subsidy >> halvings
  } else {
    0
  }
}
