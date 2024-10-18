use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Sign {
  #[arg(long, help = "Sign for <ADDRESS>.")]
  address: Address<NetworkUnchecked>,
  #[arg(long, help = "Sign with <MESSAGE>.")]
  message: String,
}

impl Sign {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let to_spend = bip322::create_to_spend(
      &self.address.require_network(wallet.chain().network())?,
      self.message.as_bytes(),
    )?;

    let to_sign = bip322::create_to_sign(&to_spend, None)?;

    todo!()
  }
}
