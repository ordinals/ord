use super::*;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[clap(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(Export),
  #[clap(about = "Update the index")]
  Run,
}

impl IndexSubcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Export(export) => export.run(options),
      Self::Run => index::run(options),
    }
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[clap(
    long,
    default_value = "inscription_number_to_id.tsv",
    help = "<TSV> file to write to"
  )]
  tsv: String,
  #[clap(long, help = "Whether to include addresses in export")]
  include_addresses: bool,
}

impl Export {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;
    index.export(&self.tsv, self.include_addresses)?;

    Ok(())
  }
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;

  index.update()?;

  Ok(())
}
