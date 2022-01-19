use super::*;

pub(crate) fn run(needle: &str) -> Result {
  for c in needle.chars() {
    if !('a'..='z').contains(&c) {
      return Err("Invalid name".into());
    }
  }

  let mut min = 0;
  let mut max = 2099999997690000;
  let mut guess = max / 2;

  loop {
    log::info!("min max guess: {} {} {}", min, max, guess);

    let name = Ordinal::new(guess).name();

    match name
      .len()
      .cmp(&needle.len())
      .then(name.deref().cmp(needle))
      .reverse()
    {
      Ordering::Less => min = guess + 1,
      Ordering::Equal => break,
      Ordering::Greater => max = guess,
    }

    if max - min == 0 {
      return Err("Name out of range".into());
    }

    guess = min + (max - min) / 2;
  }

  println!("{}", guess);

  Ok(())
}
