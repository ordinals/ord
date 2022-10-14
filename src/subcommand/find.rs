use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Find {
  #[clap(help = "Find output and offset of <ORDINAL>.")]
  ordinal: Ordinal,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.index()?;

    match index.find(self.ordinal.0)? {
      Some(satpoint) => {
        println!("{satpoint}");
        Ok(())
      }
      None => Err(anyhow!("Ordinal has not been mined as of index height")),
    }
  }
}
