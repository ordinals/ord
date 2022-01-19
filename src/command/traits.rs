use super::*;

pub(crate) fn run(ordinal: Ordinal) -> Result {
  let n = ordinal.n();

  if n % 2 == 0 {
    println!("even");
  } else {
    println!("odd");
  }

  let isqrt = n.integer_sqrt();
  if isqrt * isqrt == n {
    println!("square");
  }

  let icbrt = n.integer_cbrt();
  if icbrt * icbrt * icbrt == n {
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
    "luck: {}/{}",
    digits.iter().filter(|c| **c == '8').count() as i64
      - digits.iter().filter(|c| **c == '4').count() as i64,
    digits.len()
  );

  println!("population: {}", ordinal.population());

  println!("name: {}", ordinal.name());

  if let Some(character) = char::from_u32((n % 0x110000) as u32) {
    println!("character: {:?}", character);
  }

  println!("epoch: {}", ordinal.epoch());

  println!("height: {}", ordinal.height());

  if ordinal.subsidy_position() == 0 {
    println!("shiny");
  }

  if ordinal.height() == 124724 {
    if ordinal == 623624999999999 {
      println!("illusive");
    } else {
      println!("cursed");
    }
  }

  Ok(())
}
