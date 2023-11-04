use super::*;

#[derive(Debug, Parser)]
pub(crate) struct DumpUtxos {
  #[arg(long, help = "Dump UTXOS to <FILE>")]
  file: PathBuf,
}

impl DumpUtxos {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    // todo:
    // - see if compression helps
    // - benchmark
    // - create and distribute snapshots to users `utxo.dump`

    let index = Index::open(&options)?;

    index.update()?;

    index.dump_utxos(File::create(self.file)?)?;

    Ok(Box::new(Empty {}))
  }
}
