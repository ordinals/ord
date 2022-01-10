use super::*;

pub(crate) fn run(n: u64) -> Result {
  if n < subsidy(0) {
    println!("genesis");
  }

  if n % 2 == 0 {
    println!("even");
  } else {
    println!("odd");
  }

  if (n as f64).sqrt().fract() == 0.0 {
    println!("square");
  }

  if (n as f64).cbrt().fract() == 0.0 {
    println!("cube");
  }

  let digits = n.to_string().chars().collect::<Vec<char>>();

  let pi = std::f64::consts::PI.to_string().replace('.', "");
  let s = n.to_string();
  if s == pi[..s.len()] {
    println!("pi");
  }

  if digits.chunks(2).all(|chunk| chunk == ['6', '9']) {
    println!("nice");
  }

  if digits.iter().all(|c| *c == '7') {
    println!("angelic");
  }

  println!(
    "luck:{}/{}",
    digits.iter().filter(|c| **c == '8').count(),
    digits.len()
  );

  println!(
    "population:{}",
    (n.wrapping_mul(0x0002000400080010) & 0x1111111111111111).wrapping_mul(0x1111111111111111)
      >> 60
  );

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
