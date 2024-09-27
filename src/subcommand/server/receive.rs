use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub addresses: Vec<Address<NetworkUnchecked>>,
}

pub(super) async fn run(
  Extension(config): Extension<Arc<ServerConfig>>,
  Path(number): Path<u32>,
) -> ServerResult {
  let wallet = config.wallet.as_ref().ok_or_else(|| anyhow!("no wallet loaded"))?;
  let mut addresses: Vec<Address<NetworkUnchecked>> = Vec::new();

  // if number is 0 or empty, set 1 as default
  for _ in 0..number.max(1) {
    addresses.push(
      wallet
        .bitcoin_client()
        .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?,
    );
  }

  Ok(Json(Output { addresses }).into_response())
}
