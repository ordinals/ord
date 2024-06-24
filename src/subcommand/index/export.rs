use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[arg(long, help = "Include addresses in export")]
  include_addresses: bool,
  #[arg(long, help = "Write export to <TSV>")]
  tsv: String,
  #[arg(long, help = "Export from sequence number <GT_SEQUENCE>")]
  gt_sequnce: u32,
}

impl Export {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;
    index.export(&self.tsv, self.gt_sequnce, true)?;

    Ok(None)
  }
}
