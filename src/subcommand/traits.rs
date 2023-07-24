use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Traits {
  #[clap(help = "Show traits for <SAT>.")]
  sat: Sat,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub number: u64,
  pub decimal: String,
  pub degree: String,
  pub name: String,
  pub height: u64,
  pub cycle: u64,
  pub epoch: u64,
  pub period: u64,
  pub offset: u64,
  pub rarity: Rarity,
}

impl Traits {
  pub(crate) fn run(self) -> Result {
    print_json(Output {
      number: self.sat.n(),
      decimal: self.sat.decimal().to_string(),
      degree: self.sat.degree().to_string(),
      name: self.sat.name(),
      height: self.sat.height().0,
      cycle: self.sat.cycle(),
      epoch: self.sat.epoch().0,
      period: self.sat.period(),
      offset: self.sat.third(),
      rarity: self.sat.rarity(),
    })?;

    Ok(())
  }
}
