use super::*;

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct WalletConstructor {
  ord_client: reqwest::blocking::Client,
  name: String,
  no_sync: bool,
  rpc_url: Url,
  settings: Settings,
}

#[allow(dead_code)]
impl WalletConstructor {
  pub(crate) fn construct(
    name: String,
    no_sync: bool,
    settings: Settings,
    rpc_url: Url,
  ) -> Result<Wallet> {
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
      no_sync,
      rpc_url,
      settings,
    }
    .build()
  }

  pub(crate) fn build(self) -> Result<Wallet> {
    let database = Arc::new(open_database(&self.name, &self.settings)?);

    let mut persister = DatabasePersister(database.clone());

    let rtx = database.begin_read()?;

    let master_private_key = rtx
      .open_table(XPRIV)?
      .get(())?
      .map(|xpriv| Xpriv::decode(xpriv.value().as_slice()))
      .transpose()?
      .ok_or(anyhow!("couldn't load master private key from database"))?;

    let descriptor = descriptor::standard(
      self.settings.chain().network(),
      master_private_key,
      0,
      false,
    )?;

    let change_descriptor =
      descriptor::standard(self.settings.chain().network(), master_private_key, 0, true)?;

    let wallet = match bdk::Wallet::load()
      .check_network(self.settings.chain().network())
      .descriptor(KeychainKind::External, Some(descriptor))
      .descriptor(KeychainKind::Internal, Some(change_descriptor))
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

  fn get_output_info(&self, outputs: Vec<OutPoint>) -> Result<BTreeMap<OutPoint, api::Output>> {
    let response = self.post("/outputs", &outputs)?;

    if !response.status().is_success() {
      bail!("wallet failed get outputs: {}", response.text()?);
    }

    let response_outputs = serde_json::from_str::<Vec<api::Output>>(&response.text()?)?;

    ensure! {
      response_outputs.len() == outputs.len(),
      "unexpected server `/outputs` response length",
    }

    let output_info: BTreeMap<OutPoint, api::Output> =
      outputs.into_iter().zip(response_outputs).collect();

    for (output, info) in &output_info {
      if !info.indexed {
        bail!("output in wallet but not in ord server: {output}");
      }
    }

    Ok(output_info)
  }

  fn get_inscriptions(
    &self,
    inscriptions: &Vec<InscriptionId>,
  ) -> Result<(
    BTreeMap<SatPoint, Vec<InscriptionId>>,
    BTreeMap<InscriptionId, api::Inscription>,
  )> {
    let response = self.post("/inscriptions", inscriptions)?;

    if !response.status().is_success() {
      bail!("wallet failed get inscriptions: {}", response.text()?);
    }

    let mut inscriptions = BTreeMap::new();
    let mut inscription_infos = BTreeMap::new();
    for info in serde_json::from_str::<Vec<api::Inscription>>(&response.text()?)? {
      inscriptions
        .entry(info.satpoint)
        .or_insert_with(Vec::new)
        .push(info.id);

      inscription_infos.insert(info.id, info);
    }

    Ok((inscriptions, inscription_infos))
  }

  fn get_utxos(bitcoin_client: &Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    Ok(
      bitcoin_client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let txout = TxOut {
            script_pubkey: utxo.script_pub_key,
            value: utxo.amount,
          };

          (outpoint, txout)
        })
        .collect(),
    )
  }

  fn get_locked_utxos(bitcoin_client: &Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: Txid,
      vout: u32,
    }

    let outpoints = bitcoin_client.call::<Vec<JsonOutPoint>>("listlockunspent", &[])?;

    let mut utxos = BTreeMap::new();

    for outpoint in outpoints {
      let Some(tx_out) = bitcoin_client.get_tx_out(&outpoint.txid, outpoint.vout, Some(false))?
      else {
        continue;
      };

      utxos.insert(
        OutPoint::new(outpoint.txid, outpoint.vout),
        TxOut {
          value: tx_out.value,
          script_pubkey: ScriptBuf::from_bytes(tx_out.script_pub_key.hex),
        },
      );
    }

    Ok(utxos)
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

  pub fn post(&self, path: &str, body: &impl Serialize) -> Result<reqwest::blocking::Response> {
    self
      .ord_client
      .post(self.rpc_url.join(path)?)
      .json(body)
      .header(reqwest::header::ACCEPT, "application/json")
      .send()
      .map_err(|err| anyhow!(err))
  }
}
