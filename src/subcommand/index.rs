use super::*;

mod export;
pub mod info;
mod rune;
mod update;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[command(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(export::Export),
  #[command(about = "Print index statistics")]
  Info(info::Info),
  #[command(about = "Update the index", alias = "run")]
  Update,
  #[command(about = "Write rune to a file")]
  Rune(rune::Rune),
}

impl IndexSubcommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self {
      Self::Export(export) => export.run(settings),
      Self::Info(info) => info.run(settings),
      Self::Update => update::run(settings),
      Self::Rune(rune) => rune.run(settings),
    }
  }
}
