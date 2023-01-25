use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Subsidy {
  #[clap(help = "List sats in subsidy at <HEIGHT>.")]
  height: Height,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub first: u64,
  pub subsidy: u64,
  pub name: String,
}

impl Subsidy {
  pub(crate) fn run(self) -> Result {
    let first = self.height.starting_sat();

    let subsidy = self.height.subsidy();

    if subsidy == 0 {
      bail!("block {} has no subsidy", self.height);
    }

    print_json(Output {
      first: first.0,
      subsidy,
      name: first.name(),
    })?;

    Ok(())
  }
}
