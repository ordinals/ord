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
  pub(crate) integration_test: bool,
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
    );

    let rpc_pass = Self::setting_opt(
      &env,
      options.bitcoin_rpc_pass.as_deref(),
      Some("BITCOIN_RPC_PASS"),
      config.bitcoin_rpc_pass.as_deref(),
    );

    let integration_test = Self::setting_opt(&env, None, Some("INTEGRATION_TEST"), None)
      .map(|value| !value.is_empty())
      .unwrap_or_default();

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
      integration_test,
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
    if self.integration_test {
      0
    } else {
      self
        .first_inscription_height
        .unwrap_or_else(|| self.chain().first_inscription_height())
    }
  }

  pub(crate) fn first_rune_height(&self) -> u32 {
    if self.integration_test {
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
    env_key: Option<&'static str>,
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
    env_key: Option<&'static str>,
    config_value: Option<&str>,
  ) -> Option<String> {
    if let Some(arg_value) = arg_value {
      return Some(arg_value.into());
    }

    if let Some(env_key) = env_key {
      if let Some(env_value) = env.get(env_key) {
        return Some(env_value.into());
      }
    }

    config_value.map(str::to_string)
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

  fn parse_wallet_args(args: &str) -> (Options, subcommand::wallet::WalletCommand) {
    match Arguments::try_parse_from(args.split_whitespace()) {
      Ok(arguments) => match arguments.subcommand {
        Subcommand::Wallet(wallet) => (arguments.options, wallet),
        subcommand => panic!("unexpected subcommand: {subcommand:?}"),
      },
      Err(err) => panic!("error parsing arguments: {err}"),
    }
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
      Settings::setting_opt(&Default::default(), None, None, None),
      None
    );

    assert_eq!(
      Settings::setting_opt(&Default::default(), None, None, Some("config")),
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
      ),
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
      ),
      Some("option".into()),
    );
  }

  #[test]
  fn rpc_url_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from([
        "ord",
        "--rpc-url=127.0.0.1:1234",
        "--chain=signet",
        "index",
        "update"
      ])
      .unwrap()
      .options
      .settings()
      .unwrap()
      .rpc_url(None),
      "127.0.0.1:1234/"
    );
  }

  #[test]
  fn cookie_file_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from([
        "ord",
        "--cookie-file=/foo/bar",
        "--chain=signet",
        "index",
        "update"
      ])
      .unwrap()
      .options
      .settings()
      .unwrap()
      .cookie_file()
      .unwrap(),
      Path::new("/foo/bar")
    );
  }

  #[test]
  fn use_default_network() {
    let settings = Arguments::try_parse_from(["ord", "index", "update"])
      .unwrap()
      .options
      .settings()
      .unwrap();

    assert_eq!(settings.rpc_url(None), "127.0.0.1:8332/");

    assert!(settings.cookie_file().unwrap().ends_with(".cookie"));
  }

  #[test]
  fn uses_network_defaults() {
    let settings = Arguments::try_parse_from(["ord", "--chain=signet", "index", "update"])
      .unwrap()
      .options
      .settings()
      .unwrap();

    assert_eq!(settings.rpc_url(None), "127.0.0.1:38332/");

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
    let cookie_file = Arguments::try_parse_from(["ord", "index", "update"])
      .unwrap()
      .options
      .settings()
      .unwrap()
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

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
    let arguments =
      Arguments::try_parse_from(["ord", "--chain=signet", "index", "update"]).unwrap();

    let cookie_file = arguments
      .options
      .settings()
      .unwrap()
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
    let arguments = Arguments::try_parse_from([
      "ord",
      "--bitcoin-data-dir=foo",
      "--chain=signet",
      "index",
      "update",
    ])
    .unwrap();

    let cookie_file = arguments
      .options
      .settings()
      .unwrap()
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
    let data_dir = Arguments::try_parse_from(["ord", "index", "update"])
      .unwrap()
      .options
      .settings()
      .unwrap()
      .data_dir()
      .display()
      .to_string();
    assert!(
      data_dir.ends_with(if cfg!(windows) { r"\ord" } else { "/ord" }),
      "{data_dir}"
    );
  }

  #[test]
  fn othernet_data_dir() {
    let data_dir = Arguments::try_parse_from(["ord", "--chain=signet", "index", "update"])
      .unwrap()
      .options
      .settings()
      .unwrap()
      .data_dir()
      .display()
      .to_string();
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
    let data_dir = Arguments::try_parse_from([
      "ord",
      "--chain=signet",
      "--data-dir",
      "foo",
      "index",
      "update",
    ])
    .unwrap()
    .options
    .settings()
    .unwrap()
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
      let data_dir = Arguments::try_parse_from(["ord", "--chain", alias, "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .data_dir()
        .display()
        .to_string();

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
    assert_eq!(
      Arguments::try_parse_from(["ord", "--signet", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .chain(),
      Chain::Signet
    );
    assert_eq!(
      Arguments::try_parse_from(["ord", "-s", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .chain(),
      Chain::Signet
    );

    Arguments::try_parse_from(["ord", "--regtest", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(
      Arguments::try_parse_from(["ord", "--regtest", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .chain(),
      Chain::Regtest
    );
    assert_eq!(
      Arguments::try_parse_from(["ord", "-r", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .chain(),
      Chain::Regtest
    );

    Arguments::try_parse_from(["ord", "--testnet", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(
      Arguments::try_parse_from(["ord", "--testnet", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .chain(),
      Chain::Testnet
    );
    assert_eq!(
      Arguments::try_parse_from(["ord", "-t", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .chain(),
      Chain::Testnet
    );
  }

  #[test]
  fn wallet_flag_overrides_default_name() {
    let (_, wallet) = parse_wallet_args("ord wallet create");
    assert_eq!(wallet.name, "ord");

    let (_, wallet) = parse_wallet_args("ord wallet --name foo create");
    assert_eq!(wallet.name, "foo")
  }

  #[test]
  fn uses_wallet_rpc() {
    let (options, _) = parse_wallet_args("ord wallet --name foo balance");

    assert_eq!(
      options.settings().unwrap().rpc_url(Some("foo".into())),
      "127.0.0.1:8332/wallet/foo"
    );
  }

  #[test]
  fn setting_db_cache_size() {
    let arguments =
      Arguments::try_parse_from(["ord", "--db-cache-size", "16000000000", "index", "update"])
        .unwrap();
    assert_eq!(arguments.options.db_cache_size, Some(16000000000));
  }

  #[test]
  fn index_runes_only_returns_true_if_index_runes_flag_is_passed_and_not_on_mainnnet() {
    assert!(Arguments::try_parse_from([
      "ord",
      "--chain=signet",
      "--index-runes",
      "index",
      "update"
    ])
    .unwrap()
    .options
    .settings()
    .unwrap()
    .index_runes());

    assert!(
      !Arguments::try_parse_from(["ord", "--index-runes", "index", "update"])
        .unwrap()
        .options
        .settings()
        .unwrap()
        .index_runes()
    );

    assert!(!Arguments::try_parse_from(["ord", "index", "update"])
      .unwrap()
      .options
      .settings()
      .unwrap()
      .index_runes());
  }

  #[test]
  fn chain_setting() {
    assert_eq!(
      Settings::new(
        Options {
          regtest: true,
          ..Default::default()
        },
        vec![("CHAIN".into(), "signet".into())]
          .into_iter()
          .collect(),
        Config {
          chain: Some(Chain::Testnet),
          ..Default::default()
        }
      )
      .unwrap()
      .chain(),
      Chain::Regtest,
    );

    assert_eq!(
      Settings::new(
        Default::default(),
        vec![("CHAIN".into(), "signet".into())]
          .into_iter()
          .collect(),
        Config {
          chain: Some(Chain::Testnet),
          ..Default::default()
        }
      )
      .unwrap()
      .chain(),
      Chain::Signet,
    );

    assert_eq!(
      Settings::new(
        Default::default(),
        Default::default(),
        Config {
          chain: Some(Chain::Testnet),
          ..Default::default()
        }
      )
      .unwrap()
      .chain(),
      Chain::Testnet,
    );

    assert_eq!(
      Settings::new(Default::default(), Default::default(), Default::default())
        .unwrap()
        .chain(),
      Chain::Mainnet,
    );
  }

  #[test]
  fn bitcoin_rpc_and_pass_setting() {
    assert_eq!(
      Settings::new(
        Options {
          bitcoin_rpc_user: Some("option_user".into()),
          bitcoin_rpc_pass: Some("option_pass".into()),
          ..Default::default()
        },
        vec![
          ("BITCOIN_RPC_USER".into(), "env_user".into()),
          ("BITCOIN_RPC_PASS".into(), "env_pass".into()),
        ]
        .into_iter()
        .collect(),
        Config {
          bitcoin_rpc_user: Some("config_user".into()),
          bitcoin_rpc_pass: Some("config_pass".into()),
          ..Default::default()
        }
      )
      .unwrap()
      .auth()
      .unwrap(),
      Auth::UserPass("option_user".into(), "option_pass".into()),
    );

    assert_eq!(
      Settings::new(
        Default::default(),
        vec![
          ("BITCOIN_RPC_USER".into(), "env_user".into()),
          ("BITCOIN_RPC_PASS".into(), "env_pass".into()),
        ]
        .into_iter()
        .collect(),
        Config {
          bitcoin_rpc_user: Some("config_user".into()),
          bitcoin_rpc_pass: Some("config_pass".into()),
          ..Default::default()
        }
      )
      .unwrap()
      .auth()
      .unwrap(),
      Auth::UserPass("env_user".into(), "env_pass".into()),
    );

    assert_eq!(
      Settings::new(
        Default::default(),
        Default::default(),
        Config {
          bitcoin_rpc_user: Some("config_user".into()),
          bitcoin_rpc_pass: Some("config_pass".into()),
          ..Default::default()
        }
      )
      .unwrap()
      .auth()
      .unwrap(),
      Auth::UserPass("config_user".into(), "config_pass".into()),
    );

    assert_eq!(
      Settings::new(Default::default(), Default::default(), Default::default())
        .unwrap()
        .auth,
      None,
    );
  }
}
