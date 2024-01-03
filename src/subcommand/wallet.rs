use {
  super::*,
  bitcoin::secp256k1::{
    rand::{self, RngCore},
    All, Secp256k1,
  },
  bitcoin::{
    bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint},
    Network,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  fee_rate::FeeRate,
  miniscript::descriptor::{Descriptor, DescriptorSecretKey, DescriptorXKey, Wildcard},
  reqwest::{header, Url},
  transaction_builder::TransactionBuilder,
};

pub mod balance;
pub mod cardinals;
pub mod create;
pub mod etch;
pub mod inscribe;
pub mod inscriptions;
pub mod outputs;
pub mod receive;
pub mod restore;
pub mod sats;
pub mod send;
pub mod transaction_builder;
pub mod transactions;

#[derive(Debug, Parser)]
pub(crate) struct WalletCommand {
  #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
  pub(crate) name: String,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "Get wallet balance")]
  Balance,
  #[command(about = "Create new wallet")]
  Create(create::Create),
  #[command(about = "Create rune")]
  Etch(etch::Etch),
  #[command(about = "Create inscription")]
  Inscribe(inscribe::Inscribe),
  #[command(about = "List wallet inscriptions")]
  Inscriptions,
  #[command(about = "Generate receive address")]
  Receive,
  #[command(about = "Restore wallet")]
  Restore(restore::Restore),
  #[command(about = "List wallet satoshis")]
  Sats(sats::Sats),
  #[command(about = "Send sat or inscription")]
  Send(send::Send),
  #[command(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
  #[command(about = "List all unspent outputs in wallet")]
  Outputs,
  #[command(about = "List unspent cardinal outputs in wallet")]
  Cardinals,
}

impl WalletCommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Arc::new(Index::open(&options)?);
    let handle = axum_server::Handle::new();
    LISTENERS.lock().unwrap().push(handle.clone());

    let ord_api_url: Url = "http://127.0.0.1:8080".parse().unwrap();

    {
      let options = options.clone();
      let ord_api_url = ord_api_url.clone();
      std::thread::spawn(move || {
        crate::subcommand::server::Server {
          address: ord_api_url.host_str().map(|a| a.to_string()),
          acme_domain: vec![],
          csp_origin: None,
          http_port: ord_api_url.port(),
          https_port: None,
          acme_cache: None,
          acme_contact: vec![],
          http: true,
          https: false,
          redirect_http_to_https: false,
          enable_json_api: true,
          decompress: false,
        }
        .run(options, index, handle)
        .unwrap()
      });
    }

    let wallet = Wallet {
      bitcoin_rpc_client: bitcoin_rpc_client_for_wallet(self.name.clone(), &options)?,
      chain: options.chain(),
      ord_api_url,
      ord_http_client: {
        let mut headers = header::HeaderMap::new();
        headers.insert(
          header::ACCEPT,
          header::HeaderValue::from_static("application/json"),
        );
        let builder = reqwest::blocking::ClientBuilder::new().default_headers(headers);

        builder.build()?
      },
      wallet_name: self.name.clone(),
    };

    match self.subcommand {
      Subcommand::Balance => balance::run(wallet, options),
      Subcommand::Create(create) => create.run(wallet, options),
      Subcommand::Etch(etch) => etch.run(wallet, options),
      Subcommand::Inscribe(inscribe) => inscribe.run(wallet, options),
      Subcommand::Inscriptions => inscriptions::run(wallet, options),
      Subcommand::Receive => receive::run(wallet),
      Subcommand::Restore(restore) => restore.run(wallet, options),
      Subcommand::Sats(sats) => sats.run(wallet, options),
      Subcommand::Send(send) => send.run(wallet, options),
      Subcommand::Transactions(transactions) => transactions.run(wallet),
      Subcommand::Outputs => outputs::run(wallet, options),
      Subcommand::Cardinals => cardinals::run(wallet, options),
    }
  }
}

pub(crate) struct Wallet {
  pub(crate) bitcoin_rpc_client: Client,
  pub(crate) chain: Chain,
  pub(crate) ord_api_url: Url,
  pub(crate) ord_http_client: reqwest::blocking::Client, // TODO: make async instead of blocking
  pub(crate) wallet_name: String,
}

impl Wallet {
  pub(crate) fn get_unspent_outputs(&self) -> Result<BTreeMap<OutPoint, Amount>> {
    let mut utxos = BTreeMap::new();
    utxos.extend(
      self
        .bitcoin_rpc_client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let amount = utxo.amount;

          (outpoint, amount)
        }),
    );

    let locked_utxos: BTreeSet<OutPoint> = self.get_locked_outputs()?;

    for outpoint in locked_utxos {
      utxos.insert(
        outpoint,
        Amount::from_sat(
          self
            .bitcoin_rpc_client
            .get_raw_transaction(&outpoint.txid, None)?
            .output[TryInto::<usize>::try_into(outpoint.vout).unwrap()]
          .value,
        ),
      );
    }

    for outpoint in utxos.keys() {
      if self
        .ord_http_client
        .get(
          self
            .ord_api_url
            .join(&format!("/output/{outpoint}"))
            .unwrap(),
        )
        .send()
        .unwrap()
        .status()
        .is_client_error()
      {
        return Err(anyhow!(
          "output in Bitcoin Core wallet but not in ord index: {outpoint}"
        ));
      }
    }

    Ok(utxos)
  }

  pub(crate) fn get_unspent_output_ranges(
    &self,
    index: &Index,
  ) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
    self
      .get_unspent_outputs()?
      .into_keys()
      .map(|outpoint| match index.list(outpoint)? {
        Some(List::Unspent(sat_ranges)) => Ok((outpoint, sat_ranges)),
        Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
        None => bail!("index has not seen {outpoint}"),
      })
      .collect()
  }

  pub(crate) fn get_locked_outputs(&self) -> Result<BTreeSet<OutPoint>> {
    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: bitcoin::Txid,
      vout: u32,
    }

    Ok(
      self
        .bitcoin_rpc_client
        .call::<Vec<JsonOutPoint>>("listlockunspent", &[])?
        .into_iter()
        .map(|outpoint| OutPoint::new(outpoint.txid, outpoint.vout))
        .collect(),
    )
  }

  pub(crate) fn get_change_address(&self) -> Result<Address> {
    Ok(
      self
        .bitcoin_rpc_client
        .call::<Address<NetworkUnchecked>>("getrawchangeaddress", &["bech32m".into()])
        .context("could not get change addresses from wallet")?
        .require_network(self.chain.network())?,
    )
  }

  pub(crate) fn initialize(&self, options: &Options, seed: [u8; 64]) -> Result {
    check_version(options.bitcoin_rpc_client(None)?)?.create_wallet(
      &self.wallet_name,
      None,
      Some(true),
      None,
      None,
    )?;

    let network = self.chain.network();

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
      self.derive_and_import_descriptor(
        options,
        &secp,
        (fingerprint, derivation_path.clone()),
        derived_private_key,
        change,
      )?;
    }

    Ok(())
  }

  fn derive_and_import_descriptor(
    &self,
    options: &Options,
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

    let desc = Descriptor::new_tr(public_key, None)?;

    options
      .bitcoin_rpc_client(Some(self.wallet_name.clone()))?
      .import_descriptors(ImportDescriptors {
        descriptor: desc.to_string_with_secret(&key_map),
        timestamp: Timestamp::Now,
        active: Some(true),
        range: None,
        next_index: None,
        internal: Some(change),
        label: None,
      })?;

    Ok(())
  }
}

pub(crate) fn bitcoin_rpc_client_for_wallet(
  wallet_name: String,
  options: &Options,
) -> Result<Client> {
  let client = check_version(options.bitcoin_rpc_client(Some(wallet_name.clone()))?)?;

  if !client.list_wallets()?.contains(&wallet_name) {
    client.load_wallet(&wallet_name)?;
  }

  let descriptors = client.list_descriptors(None)?.descriptors;

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

  Ok(client)
}

pub(crate) fn check_version(client: Client) -> Result<Client> {
  const MIN_VERSION: usize = 240000;

  let bitcoin_version = client.version()?;
  if bitcoin_version < MIN_VERSION {
    bail!(
      "Bitcoin Core {} or newer required, current version is {}",
      format_bitcoin_core_version(MIN_VERSION),
      format_bitcoin_core_version(bitcoin_version),
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
