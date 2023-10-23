use {super::*, crate::teleburn};

#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  #[arg(help = "Generate teleburn addresses for inscription <DESTINATION>.")]
  destination: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub ethereum: teleburn::Ethereum,
}

impl Teleburn {
  pub(crate) fn run(self) -> SubcommandResult {
    Ok(Box::new(Output {
      ethereum: self.destination.into(),
    }))
  }
}
