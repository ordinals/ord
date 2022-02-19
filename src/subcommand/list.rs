use super::*;

#[derive(Parser)]
pub(crate) struct List {
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::index(options)?;
    let ranges = index.list(self.outpoint)?;

    for (start, end) in ranges {
      println!("[{start},{end})");
    }

    Ok(())
  }
}
