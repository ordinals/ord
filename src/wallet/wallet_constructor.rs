use super::*;

#[derive(Clone)]
pub(crate) struct WalletConstructor {
  ord_client: reqwest::blocking::Client,
  name: String,
  rpc_url: Url,
  settings: Settings,
}

impl WalletConstructor {
  pub(crate) fn construct(name: String, settings: Settings, rpc_url: Url) -> Result<Wallet> {
    let mut headers = HeaderMap::new();
    headers.insert(
      reqwest::header::ACCEPT,
      reqwest::header::HeaderValue::from_static("application/json"),
    );

    if let Some((username, password)) = settings.credentials() {
      let credentials = base64_encode(format!("{username}:{password}").as_bytes());
      headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Basic {credentials}")).unwrap(),
      );
    }

    Self {
      ord_client: reqwest::blocking::ClientBuilder::new()
        .timeout(None)
        .default_headers(headers.clone())
        .build()?,
      name,
      rpc_url,
      settings,
    }
    .build()
  }

  pub(crate) fn build(self) -> Result<Wallet> {
    let database = Arc::new(Wallet::open_database(&self.settings, &self.name)?);

    let mut persister = DatabasePersister(database.clone());

    let rtx = database.begin_read()?;

    let master_private_key = rtx
      .open_table(XPRIV)?
      .get(())?
      .map(|xpriv| Xpriv::decode(xpriv.value().as_slice()))
      .transpose()?
      .ok_or(anyhow!("couldn't load master private key from database"))?;

    let wallet = match bdk::Wallet::load()
      .check_network(self.settings.chain().network())
      .descriptor(
        KeychainKind::External,
        Some(Wallet::derive_descriptor(
          self.settings.chain().network(),
          master_private_key,
          KeychainKind::External,
        )?),
      )
      .descriptor(
        KeychainKind::Internal,
        Some(Wallet::derive_descriptor(
          self.settings.chain().network(),
          master_private_key,
          KeychainKind::Internal,
        )?),
      )
      .extract_keys()
      .lookahead(1000)
      .load_wallet(&mut persister)?
    {
      Some(wallet) => wallet,
      None => bail!("no wallet found, create one first"),
    };

    let status = self.get_server_status()?;

    Ok(Wallet {
      database,
      has_rune_index: status.rune_index,
      has_sat_index: status.sat_index,
      ord_client: self.ord_client,
      rpc_url: self.rpc_url,
      settings: self.settings,
      wallet,
    })
  }

  fn get_server_status(&self) -> Result<api::Status> {
    let response = self.get("/status")?;

    if !response.status().is_success() {
      bail!("could not get status: {}", response.text()?)
    }

    Ok(serde_json::from_str(&response.text()?)?)
  }

  pub fn get(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self
      .ord_client
      .get(self.rpc_url.join(path)?)
      .send()
      .map_err(|err| anyhow!(err))
  }
}
