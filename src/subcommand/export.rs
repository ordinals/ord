use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[arg(
    long,
    help = "Only export inscriptions matching <CONTENT_TYPE_FILTER>."
  )]
  content_type_filter: Option<Regex>,
  #[arg(long, help = "Save inscriptions to <DIRECTORY>.")]
  directory: PathBuf,
}

#[derive(Serialize)]
struct Output {
  exported: u64,
}

impl Export {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;

    index.update()?;

    let exported = index.export_inscriptions(&self.directory, self.content_type_filter.as_ref())?;

    Ok(Some(Box::new(Output { exported })))
  }
}
