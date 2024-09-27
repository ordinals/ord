use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PendingOutput {
  pub commit: Txid,
  pub rune: SpacedRune,
}

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Wallet>>>>,
) -> ServerResult {
  let wallet = wallet.lock().unwrap();

  if let Some(wallet) = wallet.as_ref() {
    let etchings: Vec<PendingOutput> = wallet
      .pending_etchings()?
      .into_iter()
      .map(|(_, entry)| {
        let spaced_rune = entry.output.rune.unwrap().rune;

        PendingOutput {
          rune: spaced_rune,
          commit: entry.commit.txid(),
        }
      })
      .collect::<Vec<PendingOutput>>();

    Ok(Json(etchings).into_response())
  } else {
    eprintln!("no wallet loaded");
    return Err(anyhow!("no wallet loaded").into());
  }
}
