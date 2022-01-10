use super::*;

pub(crate) fn run(n: u64) -> Result {
  if n == 0 {
    println!("zero");
  }

  if n < subsidy(0) {
    println!("genesis");
  }

  if n % 2 == 0 {
    println!("even");
  } else {
    println!("odd");
  }

  Ok(())
}
