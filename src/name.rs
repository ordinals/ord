use super::*;

pub(crate) fn run(needle: &str) -> Result {
  let mut min = 0;
  let mut max = u64::max_value();
  let mut guess = max / 2;

  loop {
    let name = name(guess);

    match name.len().cmp(&needle.len()).then(name.deref().cmp(needle)) {
      Ordering::Less => min = guess,
      Ordering::Equal => break,
      Ordering::Greater => max = guess,
    }

    if max == min {
      return Err("Could not find name!".into());
    }

    guess = min + (max - min) / 2;
  }

  println!("{}", guess);

  Ok(())
}
