use super::*;

mod export;
pub mod info;
mod makeutxoidx;
mod update;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[command(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(export::Export),
  #[command(about = "Print index statistics")]
  Info(info::Info),
  #[command(about = "Update the index", alias = "run")]
  Update,
  #[command(about = "Create a UTXO index")]
  UTXO(makeutxoidx::MakeUTXOIdx),
}

impl IndexSubcommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self {
      Self::Export(export) => export.run(settings),
      Self::Info(info) => info.run(settings),
      Self::UTXO(makeutxoidx) => makeutxoidx.run(),
      Self::Update => update::run(settings),
    }
  }
}
