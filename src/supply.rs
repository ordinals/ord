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
  println!("first: {}", 0);
  println!("last: {}", SUPPLY - 1);
  println!("last mined in block: {}", last);

  Ok(())
}
