use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Subsidy {
  #[clap(help = "List sats in subsidy at <HEIGHT>.")]
  height: Height,
}

impl Subsidy {
  pub(crate) fn run(self) -> Result {
    let first = self.height.starting_sat();

    let subsidy = self.height.subsidy();

    if subsidy == 0 {
      bail!("block {} has no subsidy", self.height);
    }

    println!("{}\t{}\t{}", first, self.height.subsidy(), first.name());

    Ok(())
  }
}
