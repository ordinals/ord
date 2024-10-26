use {super::*, bitcoincore_rpc::Auth};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
  bitcoin_data_dir: Option<PathBuf>,
  bitcoin_rpc_limit: Option<u32>,
  bitcoin_rpc_password: Option<String>,
  bitcoin_rpc_url: Option<String>,
  bitcoin_rpc_username: Option<String>,
  chain: Option<Chain>,
  commit_interval: Option<usize>,
  config: Option<PathBuf>,
  config_dir: Option<PathBuf>,
  cookie_file: Option<PathBuf>,
  data_dir: Option<PathBuf>,
  height_limit: Option<u32>,
  hidden: Option<HashSet<InscriptionId>>,
  http_port: Option<u16>,
  index: Option<PathBuf>,
  index_addresses: bool,
  index_cache_size: Option<usize>,
  index_runes: bool,
  index_sats: bool,
  index_transactions: bool,
  integration_test: bool,
  no_index_inscriptions: bool,
  server_password: Option<String>,
  server_url: Option<String>,
  server_username: Option<String>,
}

impl Settings {
  pub fn load(options: Options) -> Result<Settings> {
    let mut env = BTreeMap::<String, String>::new();

    for (var, value) in env::vars_os() {
      let Some(var) = var.to_str() else {
        continue;
      };

      let Some(key) = var.strip_prefix("ORD_") else {
        continue;
      };

      env.insert(
        key.into(),
        value.into_string().map_err(|value| {
          anyhow!(
            "environment variable `{var}` not valid unicode: `{}`",
            value.to_string_lossy()
          )
        })?,
      );
    }

    Self::merge(options, env)
  }

  pub fn merge(options: Options, env: BTreeMap<String, String>) -> Result<Self> {
    let settings = Settings::from_options(options).or(Settings::from_env(env)?);

    let config_path = if let Some(path) = &settings.config {
      Some(path.into())
    } else {
      let path = if let Some(dir) = settings.config_dir.clone().or(settings.data_dir.clone()) {
        dir
      } else {
        Self::default_data_dir()?
      }
      .join("ord.yaml");

      path.exists().then_some(path)
    };

    let config = if let Some(config_path) = config_path {
      serde_yaml::from_reader(fs::File::open(&config_path).context(anyhow!(
        "failed to open config file `{}`",
        config_path.display()
      ))?)
      .context(anyhow!(
        "failed to deserialize config file `{}`",
        config_path.display()
      ))?
    } else {
      Settings::default()
    };

    let settings = settings.or(config).or_defaults()?;

    match (
      &settings.bitcoin_rpc_username,
      &settings.bitcoin_rpc_password,
    ) {
      (None, Some(_rpc_pass)) => bail!("no bitcoin RPC username specified"),
      (Some(_rpc_user), None) => bail!("no bitcoin RPC password specified"),
      _ => {}
    };

    match (&settings.server_username, &settings.server_password) {
      (None, Some(_rpc_pass)) => bail!("no username specified"),
      (Some(_rpc_user), None) => bail!("no password specified"),
      _ => {}
    };

    Ok(settings)
  }

  pub fn or(self, source: Settings) -> Self {
    Self {
      bitcoin_data_dir: self.bitcoin_data_dir.or(source.bitcoin_data_dir),
      bitcoin_rpc_limit: self.bitcoin_rpc_limit.or(source.bitcoin_rpc_limit),
      bitcoin_rpc_password: self.bitcoin_rpc_password.or(source.bitcoin_rpc_password),
      bitcoin_rpc_url: self.bitcoin_rpc_url.or(source.bitcoin_rpc_url),
      bitcoin_rpc_username: self.bitcoin_rpc_username.or(source.bitcoin_rpc_username),
      chain: self.chain.or(source.chain),
      commit_interval: self.commit_interval.or(source.commit_interval),
      config: self.config.or(source.config),
      config_dir: self.config_dir.or(source.config_dir),
      cookie_file: self.cookie_file.or(source.cookie_file),
      data_dir: self.data_dir.or(source.data_dir),
      height_limit: self.height_limit.or(source.height_limit),
      hidden: Some(
        self
          .hidden
          .iter()
          .flatten()
          .chain(source.hidden.iter().flatten())
          .cloned()
          .collect(),
      ),
      http_port: self.http_port.or(source.http_port),
      index: self.index.or(source.index),
      index_addresses: self.index_addresses || source.index_addresses,
      index_cache_size: self.index_cache_size.or(source.index_cache_size),
      index_runes: self.index_runes || source.index_runes,
      index_sats: self.index_sats || source.index_sats,
      index_transactions: self.index_transactions || source.index_transactions,
      integration_test: self.integration_test || source.integration_test,
      no_index_inscriptions: self.no_index_inscriptions || source.no_index_inscriptions,
      server_password: self.server_password.or(source.server_password),
      server_url: self.server_url.or(source.server_url),
      server_username: self.server_username.or(source.server_username),
    }
  }

  pub fn from_options(options: Options) -> Self {
    Self {
      bitcoin_data_dir: options.bitcoin_data_dir,
      bitcoin_rpc_limit: options.bitcoin_rpc_limit,
      bitcoin_rpc_password: options.bitcoin_rpc_password,
      bitcoin_rpc_url: options.bitcoin_rpc_url,
      bitcoin_rpc_username: options.bitcoin_rpc_username,
      chain: options
        .signet
        .then_some(Chain::Signet)
        .or(options.regtest.then_some(Chain::Regtest))
        .or(options.testnet.then_some(Chain::Testnet))
        .or(options.chain_argument),
      commit_interval: options.commit_interval,
      config: options.config,
      config_dir: options.config_dir,
      cookie_file: options.cookie_file,
      data_dir: options.data_dir,
      height_limit: options.height_limit,
      hidden: None,
      http_port: None,
      index: options.index,
      index_addresses: options.index_addresses,
      index_cache_size: options.index_cache_size,
      index_runes: options.index_runes,
      index_sats: options.index_sats,
      index_transactions: options.index_transactions,
      integration_test: options.integration_test,
      no_index_inscriptions: options.no_index_inscriptions,
      server_password: options.server_password,
      server_url: None,
      server_username: options.server_username,
    }
  }

  pub fn from_env(env: BTreeMap<String, String>) -> Result<Self> {
    let get_bool = |key| {
      env
        .get(key)
        .map(|value| !value.is_empty())
        .unwrap_or_default()
    };

    let get_string = |key| env.get(key).cloned();

    let get_path = |key| env.get(key).map(PathBuf::from);

    let get_chain = |key| {
      env
        .get(key)
        .map(|chain| chain.parse::<Chain>())
        .transpose()
        .with_context(|| format!("failed to parse environment variable ORD_{key} as chain"))
    };

    let inscriptions = |key| {
      env
        .get(key)
        .map(|inscriptions| {
          inscriptions
            .split_whitespace()
            .map(|inscription_id| inscription_id.parse::<InscriptionId>())
            .collect::<Result<HashSet<InscriptionId>, inscription_id::ParseError>>()
        })
        .transpose()
        .with_context(|| {
          format!("failed to parse environment variable ORD_{key} as inscription list")
        })
    };

    let get_u16 = |key| {
      env
        .get(key)
        .map(|int| int.parse::<u16>())
        .transpose()
        .with_context(|| format!("failed to parse environment variable ORD_{key} as u16"))
    };

    let get_u32 = |key| {
      env
        .get(key)
        .map(|int| int.parse::<u32>())
        .transpose()
        .with_context(|| format!("failed to parse environment variable ORD_{key} as u32"))
    };

    let get_usize = |key| {
      env
        .get(key)
        .map(|int| int.parse::<usize>())
        .transpose()
        .with_context(|| format!("failed to parse environment variable ORD_{key} as usize"))
    };

    Ok(Self {
      bitcoin_data_dir: get_path("BITCOIN_DATA_DIR"),
      bitcoin_rpc_limit: get_u32("BITCOIN_RPC_LIMIT")?,
      bitcoin_rpc_password: get_string("BITCOIN_RPC_PASSWORD"),
      bitcoin_rpc_url: get_string("BITCOIN_RPC_URL"),
      bitcoin_rpc_username: get_string("BITCOIN_RPC_USERNAME"),
      chain: get_chain("CHAIN")?,
      commit_interval: get_usize("COMMIT_INTERVAL")?,
      config: get_path("CONFIG"),
      config_dir: get_path("CONFIG_DIR"),
      cookie_file: get_path("COOKIE_FILE"),
      data_dir: get_path("DATA_DIR"),
      height_limit: get_u32("HEIGHT_LIMIT")?,
      hidden: inscriptions("HIDDEN")?,
      http_port: get_u16("HTTP_PORT")?,
      index: get_path("INDEX"),
      index_addresses: get_bool("INDEX_ADDRESSES"),
      index_cache_size: get_usize("INDEX_CACHE_SIZE")?,
      index_runes: get_bool("INDEX_RUNES"),
      index_sats: get_bool("INDEX_SATS"),
      index_transactions: get_bool("INDEX_TRANSACTIONS"),
      integration_test: get_bool("INTEGRATION_TEST"),
      no_index_inscriptions: get_bool("NO_INDEX_INSCRIPTIONS"),
      server_password: get_string("SERVER_PASSWORD"),
      server_url: get_string("SERVER_URL"),
      server_username: get_string("SERVER_USERNAME"),
    })
  }

  pub fn for_env(dir: &Path, rpc_url: &str, server_url: &str) -> Self {
    Self {
      bitcoin_data_dir: Some(dir.into()),
      bitcoin_rpc_password: None,
      bitcoin_rpc_url: Some(rpc_url.into()),
      bitcoin_rpc_username: None,
      bitcoin_rpc_limit: None,
      chain: Some(Chain::Regtest),
      commit_interval: None,
      config: None,
      config_dir: None,
      cookie_file: None,
      data_dir: Some(dir.into()),
      height_limit: None,
      hidden: None,
      http_port: None,
      index: None,
      index_addresses: true,
      index_cache_size: None,
      index_runes: true,
      index_sats: true,
      index_transactions: false,
      integration_test: false,
      no_index_inscriptions: false,
      server_password: None,
      server_url: Some(server_url.into()),
      server_username: None,
    }
  }

  pub fn or_defaults(self) -> Result<Self> {
    let chain = self.chain.unwrap_or_default();

    let bitcoin_data_dir = match &self.bitcoin_data_dir {
      Some(bitcoin_data_dir) => bitcoin_data_dir.clone(),
      None => {
        if cfg!(target_os = "linux") {
          dirs::home_dir()
            .ok_or_else(|| anyhow!("failed to get cookie file path: could not get home dir"))?
            .join(".bitcoin")
        } else {
          dirs::data_dir()
            .ok_or_else(|| anyhow!("failed to get cookie file path: could not get data dir"))?
            .join("Bitcoin")
        }
      }
    };

    let cookie_file = match self.cookie_file {
      Some(cookie_file) => cookie_file,
      None => chain.join_with_data_dir(&bitcoin_data_dir).join(".cookie"),
    };

    let data_dir = chain.join_with_data_dir(match &self.data_dir {
      Some(data_dir) => data_dir.clone(),
      None => Self::default_data_dir()?,
    });

    let index = match &self.index {
      Some(path) => path.clone(),
      None => data_dir.join("index.redb"),
    };

    Ok(Self {
      bitcoin_data_dir: Some(bitcoin_data_dir),
      bitcoin_rpc_limit: Some(self.bitcoin_rpc_limit.unwrap_or(12)),
      bitcoin_rpc_password: self.bitcoin_rpc_password,
      bitcoin_rpc_url: Some(
        self
          .bitcoin_rpc_url
          .clone()
          .unwrap_or_else(|| format!("127.0.0.1:{}", chain.default_rpc_port())),
      ),
      bitcoin_rpc_username: self.bitcoin_rpc_username,
      chain: Some(chain),
      commit_interval: Some(self.commit_interval.unwrap_or(5000)),
      config: None,
      config_dir: None,
      cookie_file: Some(cookie_file),
      data_dir: Some(data_dir),
      height_limit: self.height_limit,
      hidden: self.hidden,
      http_port: self.http_port,
      index: Some(index),
      index_addresses: self.index_addresses,
      index_cache_size: Some(match self.index_cache_size {
        Some(index_cache_size) => index_cache_size,
        None => {
          let mut sys = System::new();
          sys.refresh_memory();
          usize::try_from(sys.total_memory() / 4)?
        }
      }),
      index_runes: self.index_runes,
      index_sats: self.index_sats,
      index_transactions: self.index_transactions,
      integration_test: self.integration_test,
      no_index_inscriptions: self.no_index_inscriptions,
      server_password: self.server_password,
      server_url: self.server_url,
      server_username: self.server_username,
    })
  }

  pub fn default_data_dir() -> Result<PathBuf> {
    Ok(
      dirs::data_dir()
        .context("could not get data dir")?
        .join("ord"),
    )
  }

  pub fn bitcoin_credentials(&self) -> Result<Auth> {
    if let Some((user, pass)) = &self
      .bitcoin_rpc_username
      .as_ref()
      .zip(self.bitcoin_rpc_password.as_ref())
    {
      Ok(Auth::UserPass((*user).clone(), (*pass).clone()))
    } else {
      Ok(Auth::CookieFile(self.cookie_file()?))
    }
  }

  pub fn bitcoin_rpc_client(&self, wallet: Option<String>) -> Result<Client> {
    let rpc_url = self.bitcoin_rpc_url(wallet);

    let bitcoin_credentials = self.bitcoin_credentials()?;

    log::trace!(
      "Connecting to Bitcoin Core at {}",
      self.bitcoin_rpc_url(None)
    );

    if let Auth::CookieFile(cookie_file) = &bitcoin_credentials {
      log::trace!(
        "Using credentials from cookie file at `{}`",
        cookie_file.display()
      );

      ensure!(
        cookie_file.is_file(),
        "cookie file `{}` does not exist",
        cookie_file.display()
      );
    }

    let client = Client::new(&rpc_url, bitcoin_credentials.clone()).with_context(|| {
      format!(
        "failed to connect to Bitcoin Core RPC at `{rpc_url}` with {}",
        match bitcoin_credentials {
          Auth::None => "no credentials".into(),
          Auth::UserPass(_, _) => "username and password".into(),
          Auth::CookieFile(cookie_file) => format!("cookie file at {}", cookie_file.display()),
        }
      )
    })?;

    let mut checks = 0;
    let rpc_chain = loop {
      match client.get_blockchain_info() {
        Ok(blockchain_info) => {
          break match blockchain_info.chain.to_string().as_str() {
            "bitcoin" => Chain::Mainnet,
            "testnet" => Chain::Testnet,
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

  pub fn chain(&self) -> Chain {
    self.chain.unwrap()
  }

  pub fn commit_interval(&self) -> usize {
    self.commit_interval.unwrap()
  }

  pub fn cookie_file(&self) -> Result<PathBuf> {
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

    let path = self.chain().join_with_data_dir(path);

    Ok(path.join(".cookie"))
  }

  pub fn credentials(&self) -> Option<(&str, &str)> {
    self
      .server_username
      .as_deref()
      .zip(self.server_password.as_deref())
  }

  pub fn data_dir(&self) -> PathBuf {
    self.data_dir.as_ref().unwrap().into()
  }

  pub fn first_inscription_height(&self) -> u32 {
    if self.integration_test {
      0
    } else {
      self.chain.unwrap().first_inscription_height()
    }
  }

  pub fn first_rune_height(&self) -> u32 {
    if self.integration_test {
      0
    } else {
      self.chain.unwrap().first_rune_height()
    }
  }

  pub fn height_limit(&self) -> Option<u32> {
    self.height_limit
  }

  pub fn index(&self) -> &Path {
    self.index.as_ref().unwrap()
  }

  pub fn index_addresses_raw(&self) -> bool {
    self.index_addresses
  }

  pub fn index_inscriptions_raw(&self) -> bool {
    !self.no_index_inscriptions
  }

  pub fn index_runes_raw(&self) -> bool {
    self.index_runes
  }

  pub fn index_cache_size(&self) -> usize {
    self.index_cache_size.unwrap()
  }

  pub fn index_sats_raw(&self) -> bool {
    self.index_sats
  }

  pub fn index_transactions_raw(&self) -> bool {
    self.index_transactions
  }

  pub fn integration_test(&self) -> bool {
    self.integration_test
  }

  pub fn is_hidden(&self, inscription_id: InscriptionId) -> bool {
    self
      .hidden
      .as_ref()
      .map(|hidden| hidden.contains(&inscription_id))
      .unwrap_or_default()
  }

  pub fn bitcoin_rpc_url(&self, wallet_name: Option<String>) -> String {
    let base_url = self.bitcoin_rpc_url.as_ref().unwrap();
    match wallet_name {
      Some(wallet_name) => format!("{base_url}/wallet/{wallet_name}"),
      None => format!("{base_url}/"),
    }
  }

  pub fn bitcoin_rpc_limit(&self) -> u32 {
    self.bitcoin_rpc_limit.unwrap()
  }

  pub fn server_url(&self) -> Option<&str> {
    self.server_url.as_deref()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(args: &[&str]) -> Settings {
    let args = iter::once("ord")
      .chain(args.iter().copied())
      .collect::<Vec<&str>>();
    Settings::from_options(Options::try_parse_from(args).unwrap())
      .or_defaults()
      .unwrap()
  }

  fn wallet(args: &str) -> (Settings, subcommand::wallet::WalletCommand) {
    match Arguments::try_parse_from(args.split_whitespace()) {
      Ok(arguments) => match arguments.subcommand {
        Subcommand::Wallet(wallet) => (
          Settings::from_options(arguments.options)
            .or_defaults()
            .unwrap(),
          wallet,
        ),
        subcommand => panic!("unexpected subcommand: {subcommand:?}"),
      },
      Err(err) => panic!("error parsing arguments: {err}"),
    }
  }

  #[test]
  fn auth_missing_rpc_pass_is_an_error() {
    assert_eq!(
      Settings::merge(
        Options {
          bitcoin_rpc_username: Some("foo".into()),
          ..default()
        },
        Default::default(),
      )
      .unwrap_err()
      .to_string(),
      "no bitcoin RPC password specified"
    );
  }

  #[test]
  fn auth_missing_rpc_user_is_an_error() {
    assert_eq!(
      Settings::merge(
        Options {
          bitcoin_rpc_password: Some("foo".into()),
          ..default()
        },
        Default::default(),
      )
      .unwrap_err()
      .to_string(),
      "no bitcoin RPC username specified"
    );
  }

  #[test]
  fn auth_with_user_and_pass() {
    assert_eq!(
      parse(&["--bitcoin-rpc-username=foo", "--bitcoin-rpc-password=bar"])
        .bitcoin_credentials()
        .unwrap(),
      Auth::UserPass("foo".into(), "bar".into())
    );
  }

  #[test]
  fn auth_with_cookie_file() {
    assert_eq!(
      parse(&["--cookie-file=/var/lib/Bitcoin/.cookie"])
        .bitcoin_credentials()
        .unwrap(),
      Auth::CookieFile("/var/lib/Bitcoin/.cookie".into())
    );
  }

  #[test]
  fn cookie_file_does_not_exist_error() {
    assert_eq!(
      parse(&["--cookie-file=/foo/bar/baz/qux/.cookie"])
        .bitcoin_rpc_client(None)
        .err()
        .unwrap()
        .to_string(),
      "cookie file `/foo/bar/baz/qux/.cookie` does not exist"
    );
  }

  #[test]
  fn rpc_server_chain_must_match() {
    let core = mockcore::builder().network(Network::Testnet).build();

    let settings = parse(&[
      "--cookie-file",
      core.cookie_file().to_str().unwrap(),
      "--bitcoin-rpc-url",
      &core.url(),
    ]);

    assert_eq!(
      settings.bitcoin_rpc_client(None).unwrap_err().to_string(),
      "Bitcoin RPC server is on testnet but ord is on mainnet"
    );
  }

  #[test]
  fn rpc_url_overrides_network() {
    assert_eq!(
      parse(&["--bitcoin-rpc-url=127.0.0.1:1234", "--chain=signet"]).bitcoin_rpc_url(None),
      "127.0.0.1:1234/"
    );
  }

  #[test]
  fn cookie_file_overrides_network() {
    assert_eq!(
      parse(&["--cookie-file=/foo/bar", "--chain=signet"])
        .cookie_file()
        .unwrap(),
      Path::new("/foo/bar")
    );
  }

  #[test]
  fn use_default_network() {
    let settings = parse(&[]);

    assert_eq!(settings.bitcoin_rpc_url(None), "127.0.0.1:8332/");

    assert!(settings.cookie_file().unwrap().ends_with(".cookie"));
  }

  #[test]
  fn uses_network_defaults() {
    let settings = parse(&["--chain=signet"]);

    assert_eq!(settings.bitcoin_rpc_url(None), "127.0.0.1:38332/");

    assert!(settings
      .cookie_file()
      .unwrap()
      .display()
      .to_string()
      .ends_with(if cfg!(windows) {
        r"\signet\.cookie"
      } else {
        "/signet/.cookie"
      }));
  }

  #[test]
  fn mainnet_cookie_file_path() {
    let cookie_file = parse(&[]).cookie_file().unwrap().display().to_string();

    assert!(cookie_file.ends_with(if cfg!(target_os = "linux") {
      "/.bitcoin/.cookie"
    } else if cfg!(windows) {
      r"\Bitcoin\.cookie"
    } else {
      "/Bitcoin/.cookie"
    }))
  }

  #[test]
  fn othernet_cookie_file_path() {
    let cookie_file = parse(&["--chain=signet"])
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    assert!(cookie_file.ends_with(if cfg!(target_os = "linux") {
      "/.bitcoin/signet/.cookie"
    } else if cfg!(windows) {
      r"\Bitcoin\signet\.cookie"
    } else {
      "/Bitcoin/signet/.cookie"
    }));
  }

  #[test]
  fn cookie_file_defaults_to_bitcoin_data_dir() {
    let cookie_file = parse(&["--bitcoin-data-dir=foo", "--chain=signet"])
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    assert!(cookie_file.ends_with(if cfg!(windows) {
      r"foo\signet\.cookie"
    } else {
      "foo/signet/.cookie"
    }));
  }

  #[test]
  fn mainnet_data_dir() {
    let data_dir = parse(&[]).data_dir().display().to_string();
    assert!(
      data_dir.ends_with(if cfg!(windows) { r"\ord" } else { "/ord" }),
      "{data_dir}"
    );
  }

  #[test]
  fn othernet_data_dir() {
    let data_dir = parse(&["--chain=signet"]).data_dir().display().to_string();
    assert!(
      data_dir.ends_with(if cfg!(windows) {
        r"\ord\signet"
      } else {
        "/ord/signet"
      }),
      "{data_dir}"
    );
  }

  #[test]
  fn network_is_joined_with_data_dir() {
    let data_dir = parse(&["--chain=signet", "--datadir=foo"])
      .data_dir()
      .display()
      .to_string();
    assert!(
      data_dir.ends_with(if cfg!(windows) {
        r"foo\signet"
      } else {
        "foo/signet"
      }),
      "{data_dir}"
    );
  }

  #[test]
  fn network_accepts_aliases() {
    fn check_network_alias(alias: &str, suffix: &str) {
      let data_dir = parse(&["--chain", alias]).data_dir().display().to_string();

      assert!(data_dir.ends_with(suffix), "{data_dir}");
    }

    check_network_alias("main", "ord");
    check_network_alias("mainnet", "ord");
    check_network_alias(
      "regtest",
      if cfg!(windows) {
        r"ord\regtest"
      } else {
        "ord/regtest"
      },
    );
    check_network_alias(
      "signet",
      if cfg!(windows) {
        r"ord\signet"
      } else {
        "ord/signet"
      },
    );
    check_network_alias(
      "test",
      if cfg!(windows) {
        r"ord\testnet3"
      } else {
        "ord/testnet3"
      },
    );
    check_network_alias(
      "testnet",
      if cfg!(windows) {
        r"ord\testnet3"
      } else {
        "ord/testnet3"
      },
    );
  }

  #[test]
  fn chain_flags() {
    Arguments::try_parse_from(["ord", "--signet", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(parse(&["--signet"]).chain(), Chain::Signet);
    assert_eq!(parse(&["-s"]).chain(), Chain::Signet);

    Arguments::try_parse_from(["ord", "--regtest", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(parse(&["--regtest"]).chain(), Chain::Regtest);
    assert_eq!(parse(&["-r"]).chain(), Chain::Regtest);

    Arguments::try_parse_from(["ord", "--testnet", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(parse(&["--testnet"]).chain(), Chain::Testnet);
    assert_eq!(parse(&["-t"]).chain(), Chain::Testnet);
  }

  #[test]
  fn wallet_flag_overrides_default_name() {
    assert_eq!(wallet("ord wallet create").1.name, "ord");
    assert_eq!(wallet("ord wallet --name foo create").1.name, "foo")
  }

  #[test]
  fn uses_wallet_rpc() {
    let (settings, _) = wallet("ord wallet --name foo balance");

    assert_eq!(
      settings.bitcoin_rpc_url(Some("foo".into())),
      "127.0.0.1:8332/wallet/foo"
    );
  }

  #[test]
  fn setting_index_cache_size() {
    assert_eq!(
      parse(&["--index-cache-size=16000000000",]).index_cache_size(),
      16000000000
    );
  }

  #[test]
  fn setting_commit_interval() {
    let arguments =
      Arguments::try_parse_from(["ord", "--commit-interval", "500", "index", "update"]).unwrap();
    assert_eq!(arguments.options.commit_interval, Some(500));
  }

  #[test]
  fn index_runes() {
    assert!(parse(&["--chain=signet", "--index-runes"]).index_runes_raw());
    assert!(parse(&["--index-runes"]).index_runes_raw());
    assert!(!parse(&[]).index_runes_raw());
  }

  #[test]
  fn bitcoin_rpc_and_pass_setting() {
    let config = Settings {
      bitcoin_rpc_username: Some("config_user".into()),
      bitcoin_rpc_password: Some("config_pass".into()),
      ..default()
    };

    let tempdir = TempDir::new().unwrap();

    let config_path = tempdir.path().join("ord.yaml");

    fs::write(&config_path, serde_yaml::to_string(&config).unwrap()).unwrap();

    assert_eq!(
      Settings::merge(
        Options {
          bitcoin_rpc_username: Some("option_user".into()),
          bitcoin_rpc_password: Some("option_pass".into()),
          config: Some(config_path.clone()),
          ..default()
        },
        vec![
          ("BITCOIN_RPC_USERNAME".into(), "env_user".into()),
          ("BITCOIN_RPC_PASSWORD".into(), "env_pass".into()),
        ]
        .into_iter()
        .collect(),
      )
      .unwrap()
      .bitcoin_credentials()
      .unwrap(),
      Auth::UserPass("option_user".into(), "option_pass".into()),
    );

    assert_eq!(
      Settings::merge(
        Options {
          config: Some(config_path.clone()),
          ..default()
        },
        vec![
          ("BITCOIN_RPC_USERNAME".into(), "env_user".into()),
          ("BITCOIN_RPC_PASSWORD".into(), "env_pass".into()),
        ]
        .into_iter()
        .collect(),
      )
      .unwrap()
      .bitcoin_credentials()
      .unwrap(),
      Auth::UserPass("env_user".into(), "env_pass".into()),
    );

    assert_eq!(
      Settings::merge(
        Options {
          config: Some(config_path),
          ..default()
        },
        Default::default(),
      )
      .unwrap()
      .bitcoin_credentials()
      .unwrap(),
      Auth::UserPass("config_user".into(), "config_pass".into()),
    );

    assert_matches!(
      Settings::merge(Default::default(), Default::default())
        .unwrap()
        .bitcoin_credentials()
        .unwrap(),
      Auth::CookieFile(_),
    );
  }

  #[test]
  fn example_config_file_is_valid() {
    let _: Settings = serde_yaml::from_reader(fs::File::open("ord.yaml").unwrap()).unwrap();
  }

  #[test]
  fn from_env() {
    let env = vec![
      ("BITCOIN_DATA_DIR", "/bitcoin/data/dir"),
      ("BITCOIN_RPC_LIMIT", "12"),
      ("BITCOIN_RPC_PASSWORD", "bitcoin password"),
      ("BITCOIN_RPC_URL", "url"),
      ("BITCOIN_RPC_USERNAME", "bitcoin username"),
      ("CHAIN", "signet"),
      ("COMMIT_INTERVAL", "1"),
      ("CONFIG", "config"),
      ("CONFIG_DIR", "config dir"),
      ("COOKIE_FILE", "cookie file"),
      ("DATA_DIR", "/data/dir"),
      ("HEIGHT_LIMIT", "3"),
      ("HIDDEN", "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 703e5f7c49d82aab99e605af306b9a30e991e57d42f982908a962a81ac439832i0"),
    ("HTTP_PORT", "8080"),
      ("INDEX", "index"),
      ("INDEX_CACHE_SIZE", "4"),
      ("INDEX_ADDRESSES", "1"),
      ("INDEX_RUNES", "1"),
      ("INDEX_SATS", "1"),
      ("INDEX_TRANSACTIONS", "1"),
      ("INTEGRATION_TEST", "1"),
      ("NO_INDEX_INSCRIPTIONS", "1"),
      ("SERVER_PASSWORD", "server password"),
      ("SERVER_URL", "server url"),
      ("SERVER_USERNAME", "server username"),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value.into()))
    .collect::<BTreeMap<String, String>>();

    pretty_assert_eq!(
      Settings::from_env(env).unwrap(),
      Settings {
        bitcoin_data_dir: Some("/bitcoin/data/dir".into()),
        bitcoin_rpc_limit: Some(12),
        bitcoin_rpc_password: Some("bitcoin password".into()),
        bitcoin_rpc_url: Some("url".into()),
        bitcoin_rpc_username: Some("bitcoin username".into()),
        chain: Some(Chain::Signet),
        commit_interval: Some(1),
        config: Some("config".into()),
        config_dir: Some("config dir".into()),
        cookie_file: Some("cookie file".into()),
        data_dir: Some("/data/dir".into()),
        height_limit: Some(3),
        hidden: Some(
          vec![
            "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0"
              .parse()
              .unwrap(),
            "703e5f7c49d82aab99e605af306b9a30e991e57d42f982908a962a81ac439832i0"
              .parse()
              .unwrap()
          ]
          .into_iter()
          .collect()
        ),
        http_port: Some(8080),
        index: Some("index".into()),
        index_addresses: true,
        index_cache_size: Some(4),
        index_runes: true,
        index_sats: true,
        index_transactions: true,
        integration_test: true,
        no_index_inscriptions: true,
        server_password: Some("server password".into()),
        server_url: Some("server url".into()),
        server_username: Some("server username".into()),
      }
    );
  }

  #[test]
  fn from_options() {
    pretty_assert_eq!(
      Settings::from_options(
        Options::try_parse_from([
          "ord",
          "--bitcoin-data-dir=/bitcoin/data/dir",
          "--bitcoin-rpc-limit=12",
          "--bitcoin-rpc-password=bitcoin password",
          "--bitcoin-rpc-url=url",
          "--bitcoin-rpc-username=bitcoin username",
          "--chain=signet",
          "--commit-interval=1",
          "--config=config",
          "--config-dir=config dir",
          "--cookie-file=cookie file",
          "--datadir=/data/dir",
          "--height-limit=3",
          "--index-addresses",
          "--index-cache-size=4",
          "--index-runes",
          "--index-sats",
          "--index-transactions",
          "--index=index",
          "--integration-test",
          "--no-index-inscriptions",
          "--server-password=server password",
          "--server-username=server username",
        ])
        .unwrap()
      ),
      Settings {
        bitcoin_data_dir: Some("/bitcoin/data/dir".into()),
        bitcoin_rpc_limit: Some(12),
        bitcoin_rpc_password: Some("bitcoin password".into()),
        bitcoin_rpc_url: Some("url".into()),
        bitcoin_rpc_username: Some("bitcoin username".into()),
        chain: Some(Chain::Signet),
        commit_interval: Some(1),
        config: Some("config".into()),
        config_dir: Some("config dir".into()),
        cookie_file: Some("cookie file".into()),
        data_dir: Some("/data/dir".into()),
        height_limit: Some(3),
        hidden: None,
        http_port: None,
        index: Some("index".into()),
        index_addresses: true,
        index_cache_size: Some(4),
        index_runes: true,
        index_sats: true,
        index_transactions: true,
        integration_test: true,
        no_index_inscriptions: true,
        server_password: Some("server password".into()),
        server_url: None,
        server_username: Some("server username".into()),
      }
    );
  }

  #[test]
  fn merge() {
    let env = vec![("INDEX", "env")]
      .into_iter()
      .map(|(key, value)| (key.into(), value.into()))
      .collect::<BTreeMap<String, String>>();

    let config = Settings {
      index: Some("config".into()),
      ..default()
    };

    let tempdir = TempDir::new().unwrap();

    let config_path = tempdir.path().join("ord.yaml");

    fs::write(&config_path, serde_yaml::to_string(&config).unwrap()).unwrap();

    let options =
      Options::try_parse_from(["ord", "--config", config_path.to_str().unwrap()]).unwrap();

    pretty_assert_eq!(
      Settings::merge(options.clone(), Default::default())
        .unwrap()
        .index,
      Some("config".into()),
    );

    pretty_assert_eq!(
      Settings::merge(options, env.clone()).unwrap().index,
      Some("env".into()),
    );

    let options = Options::try_parse_from([
      "ord",
      "--index=option",
      "--config",
      config_path.to_str().unwrap(),
    ])
    .unwrap();

    pretty_assert_eq!(
      Settings::merge(options, env).unwrap().index,
      Some("option".into()),
    );
  }
}
