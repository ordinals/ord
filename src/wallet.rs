use {
  super::*,
  base64::{self, Engine},
  bitcoin::secp256k1::{All, Secp256k1},
  bitcoin::{
    bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint},
    psbt::Psbt,
    Network,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{Descriptor, ImportDescriptors, Timestamp},
  fee_rate::FeeRate,
  futures::{
    future::{self, FutureExt},
    try_join, TryFutureExt,
  },
  inscribe::ParentInfo,
  miniscript::descriptor::{DescriptorSecretKey, DescriptorXKey, Wildcard},
  reqwest::{header, Url},
  transaction_builder::TransactionBuilder,
};

pub mod inscribe;
pub mod transaction_builder;

#[derive(Clone)]
struct OrdClient {
  url: Url,
  client: reqwest::Client,
}

impl OrdClient {
  pub async fn get(&self, path: &str) -> Result<reqwest::Response> {
    let url = self.url.join(path)?;
    self
      .client
      .get(url)
      .send()
      .map_err(|err| anyhow!(err))
      .await
  }
}

pub(crate) struct Wallet {
  bitcoin_client: bitcoincore_rpc::Client,
  has_rune_index: bool,
  has_sat_index: bool,
  rpc_url: Url,
  utxos: BTreeMap<OutPoint, TxOut>,
  ord_client: reqwest::blocking::Client,
  inscription_info: BTreeMap<InscriptionId, api::Inscription>,
  output_info: BTreeMap<OutPoint, api::Output>,
  inscriptions: BTreeMap<SatPoint, Vec<InscriptionId>>,
  locked_utxos: BTreeMap<OutPoint, TxOut>,
  settings: Settings,
}

impl Wallet {
  pub(crate) fn build(
    name: String,
    no_sync: bool,
    settings: Settings,
    rpc_url: Url,
  ) -> Result<Self> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
      header::ACCEPT,
      header::HeaderValue::from_static("application/json"),
    );

    if let Some((username, password)) = settings.credentials() {
      let credentials =
        base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
      headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Basic {credentials}")).unwrap(),
      );
    }

    let ord_client = reqwest::blocking::ClientBuilder::new()
      .default_headers(headers.clone())
      .build()?;

    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()?
      .block_on(async move {
        let bitcoin_client = {
          let client = Self::check_version(settings.bitcoin_rpc_client(Some(name.clone()))?)?;

          if !client.list_wallets()?.contains(&name) {
            client.load_wallet(&name)?;
          }

          Self::check_descriptors(&name, client.list_descriptors(None)?.descriptors)?;

          client
        };

        let async_ord_client = OrdClient {
          url: rpc_url.clone(),
          client: reqwest::ClientBuilder::new()
            .default_headers(headers.clone())
            .build()?,
        };

        let chain_block_count = bitcoin_client.get_block_count().unwrap() + 1;

        if !no_sync {
          for i in 0.. {
            let response = async_ord_client.get("/blockcount").await?;
            if response.text().await?.parse::<u64>().unwrap() >= chain_block_count {
              break;
            } else if i == 20 {
              bail!("wallet failed to synchronize with ord server");
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
          }
        }

        let mut utxos = Self::get_utxos(&bitcoin_client)?;
        let locked_utxos = Self::get_locked_utxos(&bitcoin_client)?;
        utxos.extend(locked_utxos.clone());

        let requests = utxos
          .clone()
          .into_keys()
          .map(|output| (output, Self::get_output(&async_ord_client, output)))
          .collect::<Vec<(OutPoint, _)>>();

        let futures = requests.into_iter().map(|(output, req)| async move {
          let result = req.await;
          (output, result)
        });

        let results = future::join_all(futures).await;

        let mut output_info = BTreeMap::new();
        for (output, result) in results {
          let info = result?;
          output_info.insert(output, info);
        }

        let requests = output_info
          .iter()
          .flat_map(|(_output, info)| info.inscriptions.clone())
          .collect::<Vec<InscriptionId>>()
          .into_iter()
          .map(|id| (id, Self::get_inscription_info(&async_ord_client, id)))
          .collect::<Vec<(InscriptionId, _)>>();

        let futures = requests.into_iter().map(|(output, req)| async move {
          let result = req.await;
          (output, result)
        });

        let (results, status) = try_join!(
          future::join_all(futures).map(Ok),
          Self::get_server_status(&async_ord_client)
        )?;

        let mut inscriptions = BTreeMap::new();
        let mut inscription_info = BTreeMap::new();
        for (id, result) in results {
          let info = result?;
          inscriptions
            .entry(info.satpoint)
            .or_insert_with(Vec::new)
            .push(id);

          inscription_info.insert(id, info);
        }

        Ok(Wallet {
          bitcoin_client,
          has_rune_index: status.rune_index,
          has_sat_index: status.sat_index,
          inscription_info,
          inscriptions,
          locked_utxos,
          ord_client,
          output_info,
          rpc_url,
          settings,
          utxos,
        })
      })
  }

  async fn get_output(ord_client: &OrdClient, output: OutPoint) -> Result<api::Output> {
    let response = ord_client.get(&format!("/output/{output}")).await?;

    if !response.status().is_success() {
      bail!("wallet failed get output: {}", response.text().await?);
    }

    let output_json: api::Output = serde_json::from_str(&response.text().await?)?;

    if !output_json.indexed {
      bail!("output in wallet but not in ord server: {output}");
    }

    Ok(output_json)
  }

  fn get_utxos(bitcoin_client: &bitcoincore_rpc::Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    Ok(
      bitcoin_client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let txout = TxOut {
            script_pubkey: utxo.script_pub_key,
            value: utxo.amount.to_sat(),
          };

          (outpoint, txout)
        })
        .collect(),
    )
  }

  fn get_locked_utxos(
    bitcoin_client: &bitcoincore_rpc::Client,
  ) -> Result<BTreeMap<OutPoint, TxOut>> {
    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: bitcoin::Txid,
      vout: u32,
    }

    let outpoints = bitcoin_client.call::<Vec<JsonOutPoint>>("listlockunspent", &[])?;

    let mut utxos = BTreeMap::new();

    for outpoint in outpoints {
      let txout = bitcoin_client
        .get_raw_transaction(&outpoint.txid, None)?
        .output
        .get(TryInto::<usize>::try_into(outpoint.vout).unwrap())
        .cloned()
        .ok_or_else(|| anyhow!("Invalid output index"))?;

      utxos.insert(OutPoint::new(outpoint.txid, outpoint.vout), txout);
    }

    Ok(utxos)
  }

  async fn get_inscription_info(
    ord_client: &OrdClient,
    inscription_id: InscriptionId,
  ) -> Result<api::Inscription> {
    let response = ord_client
      .get(&format!("/inscription/{inscription_id}"))
      .await?;

    if !response.status().is_success() {
      bail!("inscription {inscription_id} not found");
    }

    Ok(serde_json::from_str(&response.text().await?)?)
  }

  async fn get_server_status(ord_client: &OrdClient) -> Result<api::Status> {
    let response = ord_client.get("/status").await?;

    if !response.status().is_success() {
      bail!("could not get status: {}", response.text().await?)
    }

    Ok(serde_json::from_str(&response.text().await?)?)
  }

  pub(crate) fn get_output_sat_ranges(&self) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
    ensure!(
      self.has_sat_index,
      "ord index must be built with `--index-sats` to use `--sat`"
    );

    let mut output_sat_ranges = Vec::new();
    for (output, info) in self.output_info.iter() {
      if let Some(sat_ranges) = &info.sat_ranges {
        output_sat_ranges.push((*output, sat_ranges.clone()));
      } else {
        bail!("output {output} in wallet but is spent according to ord server");
      }
    }

    Ok(output_sat_ranges)
  }

  pub(crate) fn find_sat_in_outputs(&self, sat: Sat) -> Result<SatPoint> {
    ensure!(
      self.has_sat_index,
      "ord index must be built with `--index-sats` to use `--sat`"
    );

    for (outpoint, info) in self.output_info.iter() {
      if let Some(sat_ranges) = &info.sat_ranges {
        let mut offset = 0;
        for (start, end) in sat_ranges {
          if start <= &sat.n() && &sat.n() < end {
            return Ok(SatPoint {
              outpoint: *outpoint,
              offset: offset + sat.n() - start,
            });
          }
          offset += end - start;
        }
      } else {
        continue;
      }
    }

    Err(anyhow!(format!(
      "could not find sat `{sat}` in wallet outputs"
    )))
  }

  pub(crate) fn bitcoin_client(&self) -> &bitcoincore_rpc::Client {
    &self.bitcoin_client
  }

  pub(crate) fn utxos(&self) -> &BTreeMap<OutPoint, TxOut> {
    &self.utxos
  }

  pub(crate) fn locked_utxos(&self) -> &BTreeMap<OutPoint, TxOut> {
    &self.locked_utxos
  }

  pub(crate) fn inscriptions(&self) -> &BTreeMap<SatPoint, Vec<InscriptionId>> {
    &self.inscriptions
  }

  pub(crate) fn inscription_info(&self) -> BTreeMap<InscriptionId, api::Inscription> {
    self.inscription_info.clone()
  }

  pub(crate) fn inscription_exists(&self, inscription_id: InscriptionId) -> Result<bool> {
    Ok(
      !self
        .ord_client
        .get(
          self
            .rpc_url
            .join(&format!("/inscription/{inscription_id}"))
            .unwrap(),
        )
        .send()?
        .status()
        .is_client_error(),
    )
  }

  pub(crate) fn get_parent_info(
    &self,
    parent: Option<InscriptionId>,
  ) -> Result<Option<ParentInfo>> {
    if let Some(parent_id) = parent {
      if !self.inscription_exists(parent_id)? {
        return Err(anyhow!("parent {parent_id} does not exist"));
      }

      let satpoint = self
        .inscription_info
        .get(&parent_id)
        .ok_or_else(|| anyhow!("parent {parent_id} not in wallet"))?
        .satpoint;

      let tx_out = self
        .utxos
        .get(&satpoint.outpoint)
        .ok_or_else(|| anyhow!("parent {parent_id} not in wallet"))?
        .clone();

      Ok(Some(ParentInfo {
        destination: self.get_change_address()?,
        id: parent_id,
        location: satpoint,
        tx_out,
      }))
    } else {
      Ok(None)
    }
  }

  pub(crate) fn get_runic_outputs(&self) -> Result<BTreeSet<OutPoint>> {
    let mut runic_outputs = BTreeSet::new();
    for (output, info) in self.output_info.iter() {
      if !info.runes.is_empty() {
        runic_outputs.insert(*output);
      }
    }

    Ok(runic_outputs)
  }

  pub(crate) fn get_runes_balances_for_output(
    &self,
    output: &OutPoint,
  ) -> Result<Vec<(SpacedRune, Pile)>> {
    Ok(
      self
        .output_info
        .get(output)
        .ok_or(anyhow!("output not found in wallet"))?
        .runes
        .clone(),
    )
  }

  pub(crate) fn get_rune_balance_in_output(&self, output: &OutPoint, rune: Rune) -> Result<u128> {
    Ok(
      self
        .get_runes_balances_for_output(output)?
        .iter()
        .map(|(spaced_rune, pile)| {
          if spaced_rune.rune == rune {
            pile.amount
          } else {
            0
          }
        })
        .sum(),
    )
  }

  pub(crate) fn get_rune(
    &self,
    rune: Rune,
  ) -> Result<Option<(RuneId, RuneEntry, Option<InscriptionId>)>> {
    let response = self
      .ord_client
      .get(
        self
          .rpc_url
          .join(&format!("/rune/{}", SpacedRune { rune, spacers: 0 }))
          .unwrap(),
      )
      .send()?;

    if !response.status().is_success() {
      return Ok(None);
    }

    let rune_json: api::Rune = serde_json::from_str(&response.text()?)?;

    Ok(Some((rune_json.id, rune_json.entry, rune_json.parent)))
  }

  pub(crate) fn get_change_address(&self) -> Result<Address> {
    Ok(
      self
        .bitcoin_client
        .call::<Address<NetworkUnchecked>>("getrawchangeaddress", &["bech32m".into()])
        .context("could not get change addresses from wallet")?
        .require_network(self.chain().network())?,
    )
  }

  pub(crate) fn has_sat_index(&self) -> bool {
    self.has_sat_index
  }

  pub(crate) fn has_rune_index(&self) -> bool {
    self.has_rune_index
  }

  pub(crate) fn chain(&self) -> Chain {
    self.settings.chain()
  }

  fn check_descriptors(wallet_name: &str, descriptors: Vec<Descriptor>) -> Result<Vec<Descriptor>> {
    let tr = descriptors
      .iter()
      .filter(|descriptor| descriptor.desc.starts_with("tr("))
      .count();

    let rawtr = descriptors
      .iter()
      .filter(|descriptor| descriptor.desc.starts_with("rawtr("))
      .count();

    if tr != 2 || descriptors.len() != 2 + rawtr {
      bail!("wallet \"{}\" contains unexpected output descriptors, and does not appear to be an `ord` wallet, create a new wallet with `ord wallet create`", wallet_name);
    }

    Ok(descriptors)
  }

  pub(crate) fn initialize_from_descriptors(
    name: String,
    settings: &Settings,
    descriptors: Vec<Descriptor>,
  ) -> Result {
    let client = Self::check_version(settings.bitcoin_rpc_client(Some(name.clone()))?)?;

    let descriptors = Self::check_descriptors(&name, descriptors)?;

    client.create_wallet(&name, None, Some(true), None, None)?;

    let descriptors = descriptors
      .into_iter()
      .map(|descriptor| ImportDescriptors {
        descriptor: descriptor.desc.clone(),
        timestamp: descriptor.timestamp,
        active: Some(true),
        range: descriptor.range.map(|(start, end)| {
          (
            usize::try_from(start).unwrap_or(0),
            usize::try_from(end).unwrap_or(0),
          )
        }),
        next_index: descriptor
          .next
          .map(|next| usize::try_from(next).unwrap_or(0)),
        internal: descriptor.internal,
        label: None,
      })
      .collect::<Vec<ImportDescriptors>>();

    client.import_descriptors(descriptors)?;

    Ok(())
  }

  pub(crate) fn initialize(name: String, settings: &Settings, seed: [u8; 64]) -> Result {
    Self::check_version(settings.bitcoin_rpc_client(None)?)?.create_wallet(
      &name,
      None,
      Some(true),
      None,
      None,
    )?;

    let network = settings.chain().network();

    let secp = Secp256k1::new();

    let master_private_key = ExtendedPrivKey::new_master(network, &seed)?;

    let fingerprint = master_private_key.fingerprint(&secp);

    let derivation_path = DerivationPath::master()
      .child(ChildNumber::Hardened { index: 86 })
      .child(ChildNumber::Hardened {
        index: u32::from(network != Network::Bitcoin),
      })
      .child(ChildNumber::Hardened { index: 0 });

    let derived_private_key = master_private_key.derive_priv(&secp, &derivation_path)?;

    for change in [false, true] {
      Self::derive_and_import_descriptor(
        name.clone(),
        settings,
        &secp,
        (fingerprint, derivation_path.clone()),
        derived_private_key,
        change,
      )?;
    }

    Ok(())
  }

  fn derive_and_import_descriptor(
    name: String,
    settings: &Settings,
    secp: &Secp256k1<All>,
    origin: (Fingerprint, DerivationPath),
    derived_private_key: ExtendedPrivKey,
    change: bool,
  ) -> Result {
    let secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
      origin: Some(origin),
      xkey: derived_private_key,
      derivation_path: DerivationPath::master().child(ChildNumber::Normal {
        index: change.into(),
      }),
      wildcard: Wildcard::Unhardened,
    });

    let public_key = secret_key.to_public(secp)?;

    let mut key_map = std::collections::HashMap::new();
    key_map.insert(public_key.clone(), secret_key);

    let descriptor = miniscript::descriptor::Descriptor::new_tr(public_key, None)?;

    settings
      .bitcoin_rpc_client(Some(name.clone()))?
      .import_descriptors(vec![ImportDescriptors {
        descriptor: descriptor.to_string_with_secret(&key_map),
        timestamp: Timestamp::Now,
        active: Some(true),
        range: None,
        next_index: None,
        internal: Some(change),
        label: None,
      }])?;

    Ok(())
  }

  pub(crate) fn check_version(client: Client) -> Result<Client> {
    const MIN_VERSION: usize = 240000;

    let bitcoin_version = client.version()?;
    if bitcoin_version < MIN_VERSION {
      bail!(
        "Bitcoin Core {} or newer required, current version is {}",
        Self::format_bitcoin_core_version(MIN_VERSION),
        Self::format_bitcoin_core_version(bitcoin_version),
      );
    } else {
      Ok(client)
    }
  }

  fn format_bitcoin_core_version(version: usize) -> String {
    format!(
      "{}.{}.{}",
      version / 10000,
      version % 10000 / 100,
      version % 100
    )
  }
}
