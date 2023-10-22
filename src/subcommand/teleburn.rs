use {super::*, crate::teleburn_address::EthereumTeleburnAddress};

#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  recipient: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  ethereum: EthereumTeleburnAddress,
}

impl Teleburn {
  pub(crate) fn run(self) -> SubcommandResult {
    Ok(Box::new(Output {
      ethereum: self.recipient.into(),
    }))
  }
}
