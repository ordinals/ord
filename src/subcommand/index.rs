use super::*;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[clap(about = "Check integrity of database file and try repairing if corrupted")]
  Check,
  #[clap(about = "Compact database")]
  Compact,
  #[clap(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(Export),
  #[clap(about = "Update the index")]
  Run,
}

impl IndexSubcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Check => index::check(options),
      Self::Compact => index::compact(options),
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
}

impl Export {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;
    index.export(&self.tsv)?;

    Ok(())
  }
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;

  index.update()?;

  Ok(())
}

pub(crate) fn check(options: Options) -> Result {
  let mut index = Index::open(&options)?;

  match index.check() {
    Ok(true) => println!("Ok"),
    Ok(false) => println!("Repaired"),
    Err(err) => println!("Corrupted: {err}"),
  }

  Ok(())
}

pub(crate) fn compact(options: Options) -> Result {
  let mut index = Index::open(&options)?;

  match index.compact()? {
    true => println!("Compaction performed"),
    false => println!("No compaction possible"),
  }

  Ok(())
}
