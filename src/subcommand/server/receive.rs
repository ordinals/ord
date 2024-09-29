use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub addresses: Vec<Address<NetworkUnchecked>>,
}

pub(super) async fn run_one(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
) -> ServerResult {
  run(Extension(wallet), Extension(settings), Path(1)).await
}

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
  Path(number): Path<u32>,
) -> ServerResult {
  let wallet = match init_wallet::init(wallet, settings).await {
    Ok(wallet) => wallet,
    Err(err) => {
        println!("Failed to initialize wallet: {:?}", err);
        return Err(anyhow!("Failed to initialize wallet").into());
    }
  };

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
