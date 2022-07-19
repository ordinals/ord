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

    println!("name: {}", self.ordinal.name());

    println!("epoch: {}", self.ordinal.epoch());

    println!("height: {}", self.ordinal.height());

    Ok(())
  }
}
