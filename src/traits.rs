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

  let pi = std::f64::consts::PI.to_string().replace('.', "");
  let s = n.to_string();
  if s == pi[..s.len()] {
    println!("pi");
  }

  if s.replace("69", "").is_empty() {
    println!("nice");
  }

  if s.replace("7", "").is_empty() {
    println!("angelic");
  }

  if s.replace("8", "").is_empty() {
    println!("lucky");
  }

  println!("name:{}", name(n));

  let mut block = 0;
  let mut mined = 0;
  loop {
    if n == mined {
      println!("shiny");
    }

    mined += subsidy(block);

    if mined > n {
      break;
    }

    block += 1;
  }
  println!("block:{}", block);

  Ok(())
}
