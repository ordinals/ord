use super::*;

pub(crate) fn run(options: Options) -> Result {
  let address = options
    .bitcoin_rpc_client_for_wallet_command(false)?
    .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

  println!("{}", address);

  Ok(())
}
