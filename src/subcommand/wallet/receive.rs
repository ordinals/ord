use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub address: Address<NetworkUnchecked>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let address = wallet
    .bitcoin_client(false)?
    .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

  Ok(Box::new(Output { address }))
}
