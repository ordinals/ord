use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PendingOutput {
  pub commit: Txid,
  pub rune: SpacedRune,
}

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

}
