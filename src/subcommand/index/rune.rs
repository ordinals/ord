use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Rune {
  #[arg(long, help = "Inscription id file")]
  input: String,
  #[arg(long, help = "Write export to <output>")]
  output: String,
}

impl Rune {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;
    index.export_rune(&self.output, &self.input)?;

    Ok(None)
  }
}
