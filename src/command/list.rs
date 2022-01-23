use super::*;

#[derive(StructOpt)]
pub(crate) struct List {
  #[structopt(long)]
  blocksdir: Option<PathBuf>,
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self) -> Result<()> {
    let index = Index::new(self.blocksdir.as_deref())?;
    let ranges = index.list(self.outpoint)?;

    for (start, end) in ranges {
      println!("[{start},{end})");
    }

    Ok(())
  }
}
