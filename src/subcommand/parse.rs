use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Parse {
  text: String,
}

impl Parse {
  pub(crate) fn run(self) -> Result {
    println!("{}", self.text.parse::<Ordinal>()?);
    Ok(())
  }
}
