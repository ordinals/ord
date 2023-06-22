use super::*;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[clap(about = "Write inscription number and id to a file")]
  Export,
  #[clap(about = "Update the index")]
  Run,
}

impl IndexSubcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Export => index::export(options),
      Self::Run => index::run(options),
    }
  }
}

pub(crate) fn export(options: Options) -> Result {
  let index = Index::open(&options)?;

  index.update()?;
  index.export()?;

  Ok(())
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;

  index.update()?;

  Ok(())
}
