use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::index(&options)?;

    match index.list(self.outpoint)? {
      Some(ranges) => {
        for (start, end) in ranges {
          println!("[{start},{end})");
        }
        Ok(())
      }
      None => Err(anyhow!("Output not found")),
    }
  }
}
