use super::*;

mod dump_utxos;
mod export;
mod update;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[command(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(export::Export),
  #[command(about = "Create a snapshot")]
  DumpUtxos(dump_utxos::DumpUtxos),
  #[command(about = "Update the index", alias = "run")]
  Update,
}

impl IndexSubcommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self {
      Self::Export(export) => export.run(options),
      Self::DumpUtxos(dump_utxos) => dump_utxos.run(options),
      Self::Update => update::run(options),
    }
  }
}
