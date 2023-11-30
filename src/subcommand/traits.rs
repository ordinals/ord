use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Traits {
  #[arg(help = "Show traits for <SAT>.")]
  sat: Sat,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub number: u64,
  pub decimal: String,
  pub degree: String,
  pub name: String,
  pub height: u32,
  pub cycle: u32,
  pub epoch: u32,
  pub period: u32,
  pub offset: u64,
  pub rarity: Rarity,
}

impl Traits {
  pub(crate) fn run(self) -> SubcommandResult {
    Ok(Box::new(Output {
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
    }))
  }
}
