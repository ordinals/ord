use super::*;

mod export;
mod update;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[command(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(export::Export),
  #[command(about = "Update the index", alias = "run")]
  Update,
}

impl IndexSubcommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self {
      Self::Export(export) => export.run(options),
      Self::Update => update::run(options),
    }
  }
}
