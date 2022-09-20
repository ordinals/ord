use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Parse {
  ordinal: Ordinal,
}

impl Parse {
  pub(crate) fn run(self) -> Result {
    println!("{}", self.ordinal);
    Ok(())
  }
}
