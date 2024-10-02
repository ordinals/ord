use super::*;

#[derive(Serialize, Deserialize)]
pub struct CardinalUtxo {
  pub output: OutPoint,
  pub amount: u64,
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
  
  let unspent_outputs = wallet.utxos();

  let inscribed_utxos = wallet
    .inscriptions()
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let runic_utxos = wallet.get_runic_outputs()?;

  let cardinal_utxos = unspent_outputs
    .iter()
    .filter_map(|(output, txout)| {
      if inscribed_utxos.contains(output) || runic_utxos.contains(output) {
        None
      } else {
        Some(CardinalUtxo {
          output: *output,
          amount: txout.value,
        })
      }
    })
    .collect::<Vec<CardinalUtxo>>();

    Ok(Json(cardinal_utxos).into_response())
}
