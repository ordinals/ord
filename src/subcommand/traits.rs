use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Traits {
  ordinal: Ordinal,
}

impl Traits {
  pub(crate) fn run(self) -> Result {
    if self.ordinal > Ordinal::LAST {
      return Err(anyhow!("Invalid ordinal"));
    }

    println!("number: {}", self.ordinal.n());
    println!(
      "decimal: {}.{}",
      self.ordinal.height(),
      self.ordinal.third()
    );

    let height = self.ordinal.height().n();
    let h = height / (CYCLE_EPOCHS * Epoch::BLOCKS);
    let m = height % Epoch::BLOCKS;
    let s = height % PERIOD_BLOCKS;
    let t = self.ordinal.third();
    println!("degree: {h}°{m}′{s}″{t}‴");

    println!("name: {}", self.ordinal.name());

    println!("height: {}", self.ordinal.height());
    println!("cycle: {}", self.ordinal.cycle());
    println!("epoch: {}", self.ordinal.epoch());
    println!("period: {}", self.ordinal.period());
    println!("offset: {}", self.ordinal.third());

    println!(
      "rarity: {}",
      if h == 0 && m == 0 && s == 0 && t == 0 {
        "mythic"
      } else if m == 0 && s == 0 && t == 0 {
        "legendary"
      } else if m == 0 && t == 0 {
        "epic"
      } else if s == 0 && t == 0 {
        "rare"
      } else if t == 0 {
        "uncommon"
      } else {
        "common"
      }
    );

    Ok(())
  }
}
