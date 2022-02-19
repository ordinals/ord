use super::*;

#[derive(StructOpt)]
pub(crate) struct List {
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self, index_size: Option<usize>) -> Result<()> {
    let index = Index::new(index_size)?;
    let ranges = index.list(self.outpoint)?;

    for (start, end) in ranges {
      println!("[{start},{end})");
    }

    Ok(())
  }
}
