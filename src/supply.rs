use super::*;

pub(crate) fn run() -> Result {
  let mut last = 0;

  loop {
    if subsidy(last + 1) == 0 {
      break;
    }
    last += 1;
  }

  println!("supply: {}", SUPPLY);
  println!("first: {}", SUPPLY - 1);
  println!("last subsidy block: {}", last);

  Ok(())
}
