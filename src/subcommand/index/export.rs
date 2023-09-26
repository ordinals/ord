use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[arg(
    long,
    default_value = "inscription_number_to_id.tsv",
    help = "<TSV> file to write to"
  )]
  tsv: String,
  #[arg(long, help = "Whether to include addresses in export")]
  include_addresses: bool,
}

impl Export {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;

    index.update()?;
    index.export(&self.tsv, self.include_addresses)?;

    Ok(Box::new(Empty {}))
  }
}
