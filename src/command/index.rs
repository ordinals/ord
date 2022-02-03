use super::*;

#[derive(StructOpt)]
pub(crate) struct Index {
  #[structopt(long)]
  blocksdir: Option<PathBuf>,
}

impl Index {
  pub(crate) fn run(self) -> Result<()> {
    crate::Index::new(self.blocksdir.as_deref())?;
    Ok(())
  }
}
