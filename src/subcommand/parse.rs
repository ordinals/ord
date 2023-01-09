use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Parse {
  #[clap(help = "Parse <OBJECT>.")]
  object: Object,
}

impl Parse {
  pub(crate) fn run(self) -> Result {
    println!("{}", self.object);
    Ok(())
  }
}
