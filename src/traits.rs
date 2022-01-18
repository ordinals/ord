use super::*;

pub(crate) fn run(ordinal: u64) -> Result {
  if ordinal > 2099999997689999 {
    return Err("Invalid ordinal".into());
  }

  if ordinal % 2 == 0 {
    println!("even");
  } else {
    println!("odd");
  }

  let isqrt = ordinal.integer_sqrt();
  if isqrt * isqrt == ordinal {
    println!("square");
  }

  let icbrt = ordinal.integer_cbrt();
  if icbrt * icbrt * icbrt == ordinal {
    println!("cube");
  }

  let digits = ordinal.to_string().chars().collect::<Vec<char>>();

  let pi = std::f64::consts::PI.to_string().replace('.', "");
  let s = ordinal.to_string();
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

  println!("population: {}", population(ordinal));

  println!("name: {}", name(ordinal));

  if let Some(character) = char::from_u32((ordinal % 0x110000) as u32) {
    println!("character: {:?}", character);
  }

  let mut block = 0;
  let mut mined = 0;
  loop {
    if ordinal == mined {
      println!("shiny");
    }

    let subsidy = subsidy(block);

    mined += subsidy;

    if mined > ordinal {
      println!("block: {}", block);
      break;
    }

    block += 1;
  }

  if ordinal == 623624999999999 {
    println!("illusive");
  } else if block == 124724 {
    println!("cursed");
  }

  Ok(())
}
