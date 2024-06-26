use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[arg(long, help = "Write export to <output>")]
  output: String,
  #[arg(long, help = "Export sequence number > <gt_sequnce>")]
  gt_sequnce: u32,
  #[arg(long, help = "Export sequence number < <lt_sequnce>")]
  lt_sequnce: u32,
}

impl Export {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;
    index.export(&self.output, self.gt_sequnce, self.lt_sequnce)?;

    Ok(None)
  }
}
