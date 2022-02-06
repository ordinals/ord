use super::*;

#[derive(StructOpt)]
pub(crate) struct Name {
  name: String,
}

impl Name {
  pub(crate) fn run(self) -> Result {
    if self.name.is_empty() || self.name.chars().any(|c| !('a'..='z').contains(&c)) {
      return Err("Invalid name".into());
    }

    let mut min = 0;
    let mut max = 2099999997690000;
    let mut guess = max / 2;

    loop {
      log::info!("min max guess: {} {} {}", min, max, guess);

      let name = Ordinal(guess).name();

      match name
        .len()
        .cmp(&self.name.len())
        .then(name.deref().cmp(&self.name))
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
}
