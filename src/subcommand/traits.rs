use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Traits {
  #[clap(help = "Show traits for <SAT>.")]
  sat: Sat,
}

impl Traits {
  pub(crate) fn run(self) -> Result {
    print_json(sat::Output {
      number: self.sat.n(),
      decimal: self.sat.decimal().to_string(),
      degree: self.sat.degree().to_string(),
      name: self.sat.name(),
      block: self.sat.height().0,
      cycle: self.sat.cycle(),
      epoch: self.sat.epoch().0,
      period: self.sat.period(),
      offset: self.sat.third(),
      rarity: self.sat.rarity(),
      percentile: self.sat.percentile(),
    })?;

    Ok(())
  }
}
