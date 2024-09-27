use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
  pub confirmations: i32,
}

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Wallet>>>>,
  Path(limit): Path<usize>,
) -> ServerResult {
  let wallet = wallet.lock().unwrap();

  if let Some(wallet) = wallet.as_ref() {
    let client = wallet.bitcoin_client();

    let mut output = Vec::new();
    for tx in client.list_transactions(
      None,
      if limit == 0 { Some(usize::MAX) } else { Some(limit) },
      None,
      None,
    )? {
      output.push(Output {
        transaction: tx.info.txid,
        confirmations: tx.info.confirmations,
      });
    }

    Ok(Json(output).into_response())
  } else {
    eprintln!("no wallet loaded");
    return Err(anyhow!("no wallet loaded").into());
  }
}

