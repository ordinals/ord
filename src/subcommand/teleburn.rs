use {super::*, crate::teleburn};

#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  recipient: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  ethereum: teleburn::Ethereum,
}

impl Teleburn {
  pub(crate) fn run(self) -> SubcommandResult {
    Ok(Box::new(Output {
      ethereum: self.recipient.into(),
    }))
  }
}
