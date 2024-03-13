use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub addresses: Vec<Address<NetworkUnchecked>>,
}

#[derive(Debug, Parser)]
pub(crate) struct Receive {
  #[arg(
    short,
    long,
    default_value_t = 1,
    help = "Generate <NUMBER> addresses."
  )]
  number: u64,
}

impl Receive {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let addresses = (0..self.number)
      .map(|_| {
        wallet
          .bitcoin_client()
          .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(Box::new(Output { addresses })))
  }
}
