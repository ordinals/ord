use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Parse {
  #[clap(help = "Parse <SAT>.")]
  sat: Sat,
}

impl Parse {
  pub(crate) fn run(self) -> Result {
    println!("{}", self.sat);
    Ok(())
  }
}
