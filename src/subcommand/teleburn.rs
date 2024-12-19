use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  #[arg(help = "Generate teleburn addresses for inscription <DESTINATION>.")]
  destination: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub ethereum: crate::teleburn::Ethereum,
}

impl Teleburn {
  pub(crate) fn run(self) -> SubcommandResult {
    Ok(Some(Box::new(Output {
      ethereum: self.destination.into(),
    })))
  }
}
