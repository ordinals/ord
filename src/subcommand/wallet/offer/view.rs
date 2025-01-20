use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub seller_address: Address<NetworkUnchecked>,
  pub inscription: InscriptionId,
}

#[derive(Debug, Parser)]
pub(crate) struct View {
  #[arg(long, help = "<PSBT> that encodes the offer.")]
  psbt: String,
}

impl View {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    todo!();
  }
}
