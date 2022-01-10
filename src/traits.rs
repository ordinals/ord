use super::*;

fn isqrt(x: u64) -> u64 {
  let mut a = 1;
  let mut b = ((x >> 5) + 8).min(65536);
  loop {
    let m = (a + b) >> 1;
    if m * m > x {
      b = m - 1;
    } else {
      a = m + 1;
    }

    if b < a {
      break;
    }
  }
  a - 1
}

fn icbrt(mut x: u64) -> u64 {
  let mut y = 0;
  let mut s = 30;
  while s >= 0 {
    y *= 2;
    let b = 3 * y * (y + 1) + 1;
    let bs = b << s;
    if x >= bs && b == (bs >> s) {
      x -= b;
      y += 1
    }
    s -= 3;
  }
  y
}

pub(crate) fn run(n: u64) -> Result {
  if n < subsidy(0) {
    println!("genesis");
  }

  if n % 2 == 0 {
    println!("even");
  } else {
    println!("odd");
  }

  if isqrt(n) * isqrt(n) == n {
    println!("square");
  }

  if icbrt(n) * icbrt(n) * icbrt(n) == n {
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

    let subsidy = subsidy(block);

    if subsidy == 0 {
      println!("block:∞");
      break;
    }

    mined += subsidy;

    if mined > n {
      println!("block:{}", block);
      break;
    }

    block += 1;
  }

  Ok(())
}
