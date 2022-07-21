use super::*;

#[derive(Parser)]
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
      self.ordinal.subsidy_position()
    );

    let height = self.ordinal.height().n();
    let c = height / (CYCLE_EPOCHS * Epoch::BLOCKS);
    let e = height % Epoch::BLOCKS;
    let p = height % PERIOD_BLOCKS;
    let o = self.ordinal.subsidy_position();
    println!("degree: {c}°{e}′{p}″{o}‴");

    println!("name: {}", self.ordinal.name());
    println!("height: {}", self.ordinal.height());
    println!("cycle: {}", self.ordinal.cycle());
    println!("epoch: {}", self.ordinal.epoch());
    println!("period: {}", self.ordinal.period());
    println!("offset: {}", self.ordinal.subsidy_position());

    println!(
      "rarity: {}",
      if c == 0 && o == 0 && p == 0 && e == 0 {
        "mythic"
      } else if o == 0 && p == 0 && e == 0 {
        "legendary"
      } else if o == 0 && e == 0 {
        "epic"
      } else if o == 0 && p == 0 {
        "rare"
      } else if o == 0 {
        "uncommon"
      } else {
        "common"
      }
    );

    Ok(())
  }
}
