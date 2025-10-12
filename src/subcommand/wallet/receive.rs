use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub addresses: Vec<Address<NetworkUnchecked>>,
}

#[derive(Debug, Parser)]
pub(crate) struct Receive {
  #[arg(short, long, help = "Generate <NUMBER> addresses.")]
  number: Option<u64>,
}

impl Receive {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let mut addresses: Vec<Address<NetworkUnchecked>> = Vec::new();

    for _ in 0..self.number.unwrap_or(1) {
      addresses.push(wallet.get_receive_address()?.into_unchecked());
    }

    Ok(Some(Box::new(Output { addresses })))
  }
}
