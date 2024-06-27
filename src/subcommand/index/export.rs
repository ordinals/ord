use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[arg(long, help = "Write export to <output>")]
  output: String,
  #[arg(long, help = "Export sequence number > <gt_sequence>")]
  gt_sequence: Option<u32>,
  #[arg(long, help = "Export sequence number < <lt_sequence>")]
  lt_sequence: Option<u32>,
}

impl Export {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;
    index.export(&self.output, self.gt_sequence, self.lt_sequence)?;

    Ok(None)
  }
}
