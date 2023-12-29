use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub address: Address<NetworkUnchecked>,
}

pub(crate) fn run(wallet: String, options: Options) -> SubcommandResult {
  let address = bitcoin_rpc_client_for_wallet_command(wallet, &options)?
    .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

  Ok(Box::new(Output { address }))
}
