use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Parse {
  #[clap(help = "Parse <OBJECT>.")]
  object: Object,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub object: Object,
}

impl Parse {
  pub(crate) fn run(self) -> Result {
    print_json(Output {
      object: self.object,
    })?;
    Ok(())
  }
}
