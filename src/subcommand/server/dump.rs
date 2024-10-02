use super::*;

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
) -> ServerResult {
  let wallet = match init_wallet::init(wallet, settings).await {
    Ok(wallet) => wallet,
    Err(err) => {
        println!("Failed to initialize wallet: {:?}", err);
        return Err(anyhow!("Failed to initialize wallet").into());
    }
  };

  eprintln!(
    "==========================================
= THIS STRING CONTAINS YOUR PRIVATE KEYS =
=        DO NOT SHARE WITH ANYONE        =
=========================================="
  );

  Ok(Json(
    wallet
      .bitcoin_client()
      .call::<ListDescriptorsResult>("listdescriptors", &[serde_json::to_value(true)?])? // `?` 추가
  ).into_response())
}
