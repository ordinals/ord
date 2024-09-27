use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
  pub confirmations: i32,
}

pub(super) async fn run(
  Extension(config): Extension<Arc<ServerConfig>>,
  Path(limit): Path<usize>,
) -> ServerResult {
  let wallet = config.wallet.as_ref().ok_or_else(|| anyhow!("no wallet loaded"))?;
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
}

