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

  if s.replace("69", "") == "" {
    println!("nice");
  }

  if s.replace("7", "") == "" {
    println!("angelic");
  }

  let mut name = String::new();
  let mut remainder = n;
  while remainder > 0 {
    name.push(
      "abcdefghijklmnopqrstuvwxyz"
        .chars()
        .nth(((remainder - 1) % 26) as usize)
        .unwrap(),
    );
    remainder = (remainder - 1) / 26;
  }
  println!("name:{}", name.chars().rev().collect::<String>());

  Ok(())
}
