use {super::*, bitcoincore_rpc::Auth};

#[derive(Default, Debug, Clone, Serialize)]
pub struct Settings {
  #[serde(serialize_with = "serialize_auth")]
  pub(crate) auth: Option<Auth>,
  pub(crate) bitcoin_data_dir: Option<PathBuf>,
  pub(crate) chain: Chain,
  pub(crate) cookie_file: Option<PathBuf>,
  pub(crate) credentials: Option<(String, String)>,
  pub(crate) data_dir: PathBuf,
  pub(crate) db_cache_size: Option<usize>,
  pub(crate) first_inscription_height: Option<u32>,
  pub(crate) height_limit: Option<u32>,
  pub(crate) hidden: HashSet<InscriptionId>,
  pub(crate) index: Option<PathBuf>,
  pub(crate) index_runes: bool,
  pub(crate) index_sats: bool,
  pub(crate) index_spent_sats: bool,
  pub(crate) index_transactions: bool,
  pub(crate) no_index_inscriptions: bool,
  pub(crate) rpc_url: Option<String>,
}

fn serialize_auth<S>(auth: &Option<Auth>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  match auth {
    Some(Auth::UserPass(user, pass)) => serializer.serialize_str(&format!("{user}:{pass}")),
    None => serializer.serialize_none(),
    _ => unreachable!(),
  }
}

impl Settings {
  pub(crate) fn new(
    options: Options,
    env: BTreeMap<String, String>,
    config: Config,
  ) -> Result<Self> {
    let chain = Self::setting(
      &env,
      options
        .signet
        .then_some(Chain::Signet)
        .or(options.regtest.then_some(Chain::Regtest))
        .or(options.testnet.then_some(Chain::Testnet))
        .or(options.chain_argument),
      Some("CHAIN"),
      config.chain,
      Chain::Mainnet,
    )?;

    let rpc_user = Self::setting_opt(
      &env,
      options.bitcoin_rpc_user.as_deref(),
      Some("BITCOIN_RPC_USER"),
      config.bitcoin_rpc_user.as_deref(),
    )?;

    let rpc_pass = Self::setting_opt(
      &Default::default(),
      options.bitcoin_rpc_pass.as_deref(),
      Some("BITCOIN_RPC_PASS"),
      config.bitcoin_rpc_pass.as_deref(),
    )?;

    let auth = match (rpc_user, rpc_pass) {
      (Some(rpc_user), Some(rpc_pass)) => Some(Auth::UserPass(rpc_user, rpc_pass)),
      (None, Some(_rpc_pass)) => bail!("no bitcoind rpc user specified"),
      (Some(_rpc_user), None) => bail!("no bitcoind rpc password specified"),
      _ => None,
    };

    Ok(Self {
      auth,
      bitcoin_data_dir: options.bitcoin_data_dir,
      chain,
      cookie_file: options.cookie_file,
      credentials: options.username.zip(options.password),
      data_dir: options.data_dir,
      db_cache_size: options.db_cache_size,
      first_inscription_height: options.first_inscription_height,
      height_limit: options.height_limit,
      hidden: config.hidden.unwrap_or_default(),
      index: options.index,
      index_runes: options.index_runes,
      index_sats: options.index_sats,
      index_spent_sats: options.index_spent_sats,
      index_transactions: options.index_transactions,
      no_index_inscriptions: options.no_index_inscriptions,
      rpc_url: options.rpc_url,
    })
  }

  pub(crate) fn auth(&self) -> Result<Auth> {
    if let Some(auth) = &self.auth {
      Ok(auth.clone())
    } else {
      Ok(Auth::CookieFile(self.cookie_file()?))
    }
  }

  pub(crate) fn bitcoin_rpc_client(&self, wallet: Option<String>) -> Result<Client> {
    let rpc_url = self.rpc_url(wallet);

    let auth = self.auth()?;

    log::info!("Connecting to Bitcoin Core at {}", self.rpc_url(None));

    if let Auth::CookieFile(cookie_file) = &auth {
      log::info!(
        "Using credentials from cookie file at `{}`",
        cookie_file.display()
      );

      ensure!(
        cookie_file.is_file(),
        "cookie file `{}` does not exist",
        cookie_file.display()
      );
    }

    let client = Client::new(&rpc_url, auth)
      .with_context(|| format!("failed to connect to Bitcoin Core RPC at `{rpc_url}`"))?;

    let mut checks = 0;
    let rpc_chain = loop {
      match client.get_blockchain_info() {
        Ok(blockchain_info) => {
          break match blockchain_info.chain.as_str() {
            "main" => Chain::Mainnet,
            "test" => Chain::Testnet,
            "regtest" => Chain::Regtest,
            "signet" => Chain::Signet,
            other => bail!("Bitcoin RPC server on unknown chain: {other}"),
          }
        }
        Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(err)))
          if err.code == -28 => {}
        Err(err) => bail!("Failed to connect to Bitcoin Core RPC at `{rpc_url}`:  {err}"),
      }

      ensure! {
        checks < 100,
        "Failed to connect to Bitcoin Core RPC at `{rpc_url}`",
      }

      checks += 1;
      thread::sleep(Duration::from_millis(100));
    };

    let ord_chain = self.chain();

    if rpc_chain != ord_chain {
      bail!("Bitcoin RPC server is on {rpc_chain} but ord is on {ord_chain}");
    }

    Ok(client)
  }

  pub(crate) fn chain(&self) -> Chain {
    self.chain
  }

  pub(crate) fn cookie_file(&self) -> Result<PathBuf> {
    if let Some(cookie_file) = &self.cookie_file {
      return Ok(cookie_file.clone());
    }

    let path = if let Some(bitcoin_data_dir) = &self.bitcoin_data_dir {
      bitcoin_data_dir.clone()
    } else if cfg!(target_os = "linux") {
      dirs::home_dir()
        .ok_or_else(|| anyhow!("failed to get cookie file path: could not get home dir"))?
        .join(".bitcoin")
    } else {
      dirs::data_dir()
        .ok_or_else(|| anyhow!("failed to get cookie file path: could not get data dir"))?
        .join("Bitcoin")
    };

    let path = self.chain().join_with_data_dir(&path);

    Ok(path.join(".cookie"))
  }

  pub(crate) fn credentials(&self) -> Option<(&str, &str)> {
    self
      .credentials
      .as_ref()
      .map(|(username, password)| (username.as_ref(), password.as_ref()))
  }

  pub(crate) fn data_dir(&self) -> PathBuf {
    self.chain().join_with_data_dir(&self.data_dir)
  }

  pub(crate) fn first_inscription_height(&self) -> u32 {
    if integration_test() {
      0
    } else {
      self
        .first_inscription_height
        .unwrap_or_else(|| self.chain().first_inscription_height())
    }
  }

  pub(crate) fn first_rune_height(&self) -> u32 {
    if integration_test() {
      0
    } else {
      self.chain().first_rune_height()
    }
  }

  pub(crate) fn index_runes(&self) -> bool {
    self.index_runes && self.chain() != Chain::Mainnet
  }

  pub(crate) fn is_hidden(&self, inscription_id: InscriptionId) -> bool {
    self.hidden.contains(&inscription_id)
  }

  pub(crate) fn rpc_url(&self, wallet_name: Option<String>) -> String {
    let base_url = self
      .rpc_url
      .clone()
      .unwrap_or(format!("127.0.0.1:{}", self.chain().default_rpc_port()));

    match wallet_name {
      Some(wallet_name) => format!("{base_url}/wallet/{wallet_name}"),
      None => format!("{base_url}/"),
    }
  }

  fn setting<T: FromStr<Err = Error>>(
    env: &BTreeMap<String, String>,
    arg_value: Option<T>,
    env_key: Option<&str>,
    config_value: Option<T>,
    default_value: T,
  ) -> Result<T> {
    if let Some(arg_value) = arg_value {
      return Ok(arg_value);
    }

    if let Some(env_key) = env_key {
      if let Some(env_value) = env.get(env_key) {
        return env_value
          .parse()
          .with_context(|| anyhow!("failed to parse {env_key}"));
      }
    }

    if let Some(config_value) = config_value {
      return Ok(config_value);
    }

    Ok(default_value)
  }

  fn setting_opt(
    env: &BTreeMap<String, String>,
    arg_value: Option<&str>,
    env_key: Option<&str>,
    config_value: Option<&str>,
  ) -> Result<Option<String>> {
    if let Some(arg_value) = arg_value {
      return Ok(Some(arg_value.into()));
    }

    if let Some(env_key) = env_key {
      if let Some(env_value) = env.get(env_key) {
        return Ok(Some(env_value.into()));
      }
    }

    Ok(config_value.map(str::to_string))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn settings(args: &[&str]) -> Settings {
    Settings::new(
      Options::try_parse_from(args).unwrap(),
      Default::default(),
      Default::default(),
    )
    .unwrap()
  }

  #[test]
  fn auth_missing_rpc_pass_is_an_error() {
    assert_eq!(
      Settings::new(
        Options {
          bitcoin_rpc_user: Some("foo".into()),
          ..Default::default()
        },
        Default::default(),
        Default::default(),
      )
      .unwrap_err()
      .to_string(),
      "no bitcoind rpc password specified"
    );
  }

  #[test]
  fn auth_missing_rpc_user_is_an_error() {
    assert_eq!(
      Settings::new(
        Options {
          bitcoin_rpc_pass: Some("foo".into()),
          ..Default::default()
        },
        Default::default(),
        Default::default(),
      )
      .unwrap_err()
      .to_string(),
      "no bitcoind rpc user specified"
    );
  }

  #[test]
  fn auth_with_user_and_pass() {
    assert_eq!(
      Settings::new(
        Options {
          bitcoin_rpc_user: Some("foo".into()),
          bitcoin_rpc_pass: Some("bar".into()),
          ..Default::default()
        },
        Default::default(),
        Default::default(),
      )
      .unwrap()
      .auth()
      .unwrap(),
      Auth::UserPass("foo".into(), "bar".into())
    );
  }

  #[test]
  fn auth_with_cookie_file() {
    let settings = Options {
      cookie_file: Some("/var/lib/Bitcoin/.cookie".into()),
      ..Default::default()
    }
    .settings()
    .unwrap();
    assert_eq!(
      settings.auth().unwrap(),
      Auth::CookieFile("/var/lib/Bitcoin/.cookie".into())
    );
  }

  #[test]
  fn cookie_file_does_not_exist_error() {
    assert_eq!(
      Options {
        cookie_file: Some("/foo/bar/baz/qux/.cookie".into()),
        ..Default::default()
      }
      .settings()
      .unwrap()
      .bitcoin_rpc_client(None)
      .map(|_| "")
      .unwrap_err()
      .to_string(),
      "cookie file `/foo/bar/baz/qux/.cookie` does not exist"
    );
  }

  #[test]
  fn rpc_server_chain_must_match() {
    let rpc_server = test_bitcoincore_rpc::builder()
      .network(Network::Testnet)
      .build();

    let settings = settings(&[
      "ord",
      "--cookie-file",
      rpc_server.cookie_file().to_str().unwrap(),
      "--rpc-url",
      &rpc_server.url(),
    ]);

    assert_eq!(
      settings.bitcoin_rpc_client(None).unwrap_err().to_string(),
      "Bitcoin RPC server is on testnet but ord is on mainnet"
    );
  }

  #[test]
  fn setting() {
    assert_eq!(
      Settings::setting(&Default::default(), None, None, None, Chain::Mainnet).unwrap(),
      Chain::Mainnet,
    );

    assert_eq!(
      Settings::setting(
        &Default::default(),
        None,
        None,
        Some(Chain::Testnet),
        Chain::Mainnet
      )
      .unwrap(),
      Chain::Testnet,
    );

    assert_eq!(
      Settings::setting(
        &vec![("CHAIN".to_string(), "signet".to_string())]
          .into_iter()
          .collect(),
        None,
        Some("CHAIN"),
        Some(Chain::Testnet),
        Chain::Mainnet
      )
      .unwrap(),
      Chain::Signet,
    );

    assert_eq!(
      Settings::setting(
        &vec![("CHAIN".to_string(), "signet".to_string())]
          .into_iter()
          .collect(),
        Some(Chain::Regtest),
        Some("CHAIN"),
        Some(Chain::Testnet),
        Chain::Mainnet
      )
      .unwrap(),
      Chain::Regtest,
    );
  }

  #[test]
  fn setting_opt() {
    assert_eq!(
      Settings::setting_opt(&Default::default(), None, None, None).unwrap(),
      None
    );

    assert_eq!(
      Settings::setting_opt(&Default::default(), None, None, Some("config")).unwrap(),
      Some("config".into()),
    );

    assert_eq!(
      Settings::setting_opt(
        &vec![("env_key".into(), "env_value".into())]
          .into_iter()
          .collect(),
        None,
        Some("env_key"),
        Some("config")
      )
      .unwrap(),
      Some("env_value".into()),
    );

    assert_eq!(
      Settings::setting_opt(
        &vec![("env_key".into(), "env_value".into())]
          .into_iter()
          .collect(),
        Some("option"),
        Some("env_key"),
        Some("config")
      )
      .unwrap(),
      Some("option".into()),
    );
  }
}
