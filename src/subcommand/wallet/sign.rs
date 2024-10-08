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



    todo!()
  }
}
