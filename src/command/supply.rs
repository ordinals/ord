use super::*;

pub(crate) fn run() -> Result {
  let mut last = 0;

  loop {
    if Height(last + 1).subsidy() == 0 {
      break;
    }
    last += 1;
  }

  println!("supply: {}", Ordinal::SUPPLY);
  println!("first: {}", 0);
  println!("last: {}", Ordinal::SUPPLY - 1);
  println!("last mined in block: {}", last);

  Ok(())
}
