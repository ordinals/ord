use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub address: Address<NetworkUnchecked>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let address = wallet
    .bitcoin_client()
    .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

  Ok(Some(Box::new(Output { address })))
}
