use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Find {
  ordinal: Ordinal,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::index(&options)?;

    match index.find(self.ordinal)? {
      Some(satpoint) => {
        println!("{satpoint}");
        Ok(())
      }
      None => Err(anyhow!("Ordinal has not been mined as of index height")),
    }
  }
}
