use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Find {
  #[clap(help = "Find output and offset of <SAT>.")]
  sat: Sat,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    match index.find(self.sat.0)? {
      Some(satpoint) => {
        println!("{satpoint}");
        Ok(())
      }
      None => Err(anyhow!("sat has not been mined as of index height")),
    }
  }
}
