use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[arg(long, help = "Write export to <output>")]
  output: String,
  #[arg(long, help = "old output")]
  input: Option<String>,
  #[arg(long, help = "Changes output")]
  changes_output: Option<String>,
  #[arg(long, help = "utxo:address map from <utxo_source>")]
  utxo_source: String,
  #[arg(long, help = "Export sequence number > <gt_sequence>")]
  gt_sequence: Option<u32>,
  #[arg(long, help = "Export sequence number < <lt_sequence>")]
  lt_sequence: Option<u32>,
}

impl Export {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;
    index.export(
      &self.output,
      self.input,
      self.changes_output,
      &self.utxo_source,
      self.gt_sequence,
      self.lt_sequence,
    )?;

    Ok(None)
  }
}
