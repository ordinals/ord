use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Parse {
  #[clap(help = "Parse <ORDINAL>.")]
  ordinal: Ordinal,
}

impl Parse {
  pub(crate) fn run(self) -> Result {
    println!("{}", self.ordinal);
    Ok(())
  }
}
