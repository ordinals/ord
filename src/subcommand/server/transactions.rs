use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
  pub confirmations: i32,
}

pub(super) async fn run_nolimit(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
) -> ServerResult {
  run(Extension(wallet), Extension(settings), Path(0)).await
}

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
  Path(limit): Path<usize>,
) -> ServerResult {
  let wallet = match init_wallet::init(wallet, settings).await {
    Ok(wallet) => wallet,
    Err(err) => {
        println!("Failed to initialize wallet: {:?}", err);
        return Err(anyhow!("Failed to initialize wallet").into());
    }
  };

  let client = wallet.bitcoin_client();

  let mut output = Vec::new();
  for tx in client.list_transactions(
    None,
    if limit == 0 { Some(u16::MAX.into()) } else { Some(limit) },
    None,
    None,
  )? {
    output.push(Output {
      transaction: tx.info.txid,
      confirmations: tx.info.confirmations,
    });
  }

  Ok(Json(output).into_response())
}

