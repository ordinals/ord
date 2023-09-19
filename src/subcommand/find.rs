use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Find {
  #[arg(help = "Find output and offset of <SAT>.")]
  sat: Sat,
  #[clap(help = "Find output and offset of all sats in the range <SAT>-<END>.")]
  end: Option<Sat>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub satpoint: SatPoint,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FindRangeOutput {
  pub start: u64,
  pub size: u64,
  pub satpoint: SatPoint,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;

    index.update()?;

    match self.end {
      Some(end) => {
        if self.sat < end {
          match index.find_range(self.sat.0, end.0)? {
            Some(result) => Ok(Box::new(result)),
            None => Err(anyhow!("range has not been mined as of index height")),
          }
        } else {
          Err(anyhow!("range is empty"))
        }
      }
      None => match index.find(self.sat.0)? {
        Some(satpoint) => Ok(Box::new(Output { satpoint })),
        None => Err(anyhow!("sat has not been mined as of index height")),
      },
    }
  }
}
