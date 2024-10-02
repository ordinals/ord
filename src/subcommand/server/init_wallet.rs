use super::*;

pub async fn init(wallet: Arc<Mutex<Option<Arc<Wallet>>>>, settings: Arc<Settings>) -> Result<Arc<Wallet>, Box<dyn std::error::Error>> {
  let mut wallet_guard = wallet.lock().await;

  task::block_in_place(|| {
    if let Some(wallet) = wallet_guard.as_ref() {
        return Ok(wallet.clone());
    }

    println!("Wallet not Initialized, Initializing new wallet...");
    let wallet = WalletConstructor::construct(
        "ord".to_string(),
        false,
        settings.as_ref().clone(),
        settings.server_url()
        .unwrap_or("http://127.0.0.1:80")
        .parse::<Url>()
        .context("invalid server URL")?,
    )?;

    let wallet_arc = Arc::new(wallet);
    *wallet_guard = Some(wallet_arc.clone());

    println!("New wallet initialized!");
    Ok(wallet_arc)
  })
}
