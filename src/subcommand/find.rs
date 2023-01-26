use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Find {
  #[clap(help = "Find output and offset of <SAT>.")]
  sat: Sat,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub satpoint: SatPoint,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;

    index.update()?;

    match index.find(self.sat.0)? {
      Some(satpoint) => Ok(Box::new(Output { satpoint })),
      None => Err(anyhow!("sat has not been mined as of index height")),
    }
  }
}
