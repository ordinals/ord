use {super::*, bitcoincore_rpc::Auth};

#[derive(Clone, Default, Debug, Parser)]
#[command(group(
  ArgGroup::new("chains")
    .required(false)
    .args(&["chain_argument", "signet", "regtest", "testnet"]),
))]
pub struct Options {
  #[arg(long, help = "Load Bitcoin Core data dir from <BITCOIN_DATA_DIR>.")]
  pub(crate) bitcoin_data_dir: Option<PathBuf>,
  #[arg(long, help = "Authenticate to Bitcoin Core RPC with <RPC_PASS>.")]
  pub(crate) bitcoin_rpc_pass: Option<String>,
  #[arg(long, help = "Authenticate to Bitcoin Core RPC as <RPC_USER>.")]
  pub(crate) bitcoin_rpc_user: Option<String>,
  #[arg(
    long = "chain",
    value_enum,
    default_value = "mainnet",
    help = "Use <CHAIN>."
  )]
  pub(crate) chain_argument: Chain,
  #[arg(long, help = "Load configuration from <CONFIG>.")]
  pub(crate) config: Option<PathBuf>,
  #[arg(long, help = "Load configuration from <CONFIG_DIR>.")]
  pub(crate) config_dir: Option<PathBuf>,
  #[arg(long, help = "Load Bitcoin Core RPC cookie file from <COOKIE_FILE>.")]
  pub(crate) cookie_file: Option<PathBuf>,
  #[arg(long, help = "Store index in <DATA_DIR>.", default_value_os_t = Options::default_data_dir())]
  pub(crate) data_dir: PathBuf,
  #[arg(
    long,
    help = "Set index cache to <DB_CACHE_SIZE> bytes. By default takes 1/4 of available RAM."
  )]
  pub(crate) db_cache_size: Option<usize>,
  #[arg(
    long,
    help = "Don't look for inscriptions below <FIRST_INSCRIPTION_HEIGHT>."
  )]
  pub(crate) first_inscription_height: Option<u32>,
  #[arg(long, help = "Limit index to <HEIGHT_LIMIT> blocks.")]
  pub(crate) height_limit: Option<u32>,
  #[arg(long, help = "Use index at <INDEX>.")]
  pub(crate) index: Option<PathBuf>,
  #[arg(
    long,
    help = "Track location of runes. RUNES ARE IN AN UNFINISHED PRE-ALPHA STATE AND SUBJECT TO CHANGE AT ANY TIME."
  )]
  pub(crate) index_runes: bool,
  #[arg(long, help = "Track location of all satoshis.")]
  pub(crate) index_sats: bool,
  #[arg(long, help = "Store transactions in index.")]
  pub(crate) index_transactions: bool,
  #[arg(
    long,
    short,
    alias = "noindex_inscriptions",
    help = "Do not index inscriptions."
  )]
  pub(crate) no_index_inscriptions: bool,
  #[arg(long, short, help = "Use regtest. Equivalent to `--chain regtest`.")]
  pub(crate) regtest: bool,
  #[arg(long, help = "Connect to Bitcoin Core RPC at <RPC_URL>.")]
  pub(crate) rpc_url: Option<String>,
  #[arg(long, short, help = "Use signet. Equivalent to `--chain signet`.")]
  pub(crate) signet: bool,
  #[arg(long, short, help = "Use testnet. Equivalent to `--chain testnet`.")]
  pub(crate) testnet: bool,
}

impl Options {
  pub(crate) fn chain(&self) -> Chain {
    if self.signet {
      Chain::Signet
    } else if self.regtest {
      Chain::Regtest
    } else if self.testnet {
      Chain::Testnet
    } else {
      self.chain_argument
    }
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

  fn default_data_dir() -> PathBuf {
    dirs::data_dir()
      .map(|dir| dir.join("ord"))
      .expect("failed to retrieve data dir")
  }

  pub(crate) fn data_dir(&self) -> PathBuf {
    self.chain().join_with_data_dir(&self.data_dir)
  }

  pub(crate) fn load_config(&self) -> Result<Config> {
    match &self.config {
      Some(path) => Ok(serde_yaml::from_reader(File::open(path)?)?),
      None => match &self.config_dir {
        Some(dir) if dir.join("ord.yaml").exists() => {
          Ok(serde_yaml::from_reader(File::open(dir.join("ord.yaml"))?)?)
        }
        Some(_) | None => Ok(Default::default()),
      },
    }
  }

  fn derive_var(
    arg_value: Option<&str>,
    env_key: Option<&str>,
    config_value: Option<&str>,
    default_value: Option<&str>,
  ) -> Result<Option<String>> {
    let env_value = match env_key {
      Some(env_key) => match env::var(format!("ORD_{env_key}")) {
        Ok(env_value) => Some(env_value),
        Err(err @ env::VarError::NotUnicode(_)) => return Err(err.into()),
        Err(env::VarError::NotPresent) => None,
      },
      None => None,
    };

    Ok(
      arg_value
        .or(env_value.as_deref())
        .or(config_value)
        .or(default_value)
        .map(str::to_string),
    )
  }

  pub(crate) fn auth(&self) -> Result<Auth> {
    let config = self.load_config()?;

    let rpc_user = Options::derive_var(
      self.bitcoin_rpc_user.as_deref(),
      Some("BITCOIN_RPC_USER"),
      config.bitcoin_rpc_user.as_deref(),
      None,
    )?;

    let rpc_pass = Options::derive_var(
      self.bitcoin_rpc_pass.as_deref(),
      Some("BITCOIN_RPC_PASS"),
      config.bitcoin_rpc_pass.as_deref(),
      None,
    )?;

    match (rpc_user, rpc_pass) {
      (Some(rpc_user), Some(rpc_pass)) => Ok(Auth::UserPass(rpc_user, rpc_pass)),
      (None, Some(_rpc_pass)) => Err(anyhow!("no bitcoind rpc user specified")),
      (Some(_rpc_user), None) => Err(anyhow!("no bitcoind rpc password specified")),
      _ => Ok(Auth::CookieFile(self.cookie_file()?)),
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
      .with_context(|| format!("failed to connect to Bitcoin Core RPC at {rpc_url}"))?;

    let rpc_chain = match client.get_blockchain_info()?.chain.as_str() {
      "main" => Chain::Mainnet,
      "test" => Chain::Testnet,
      "regtest" => Chain::Regtest,
      "signet" => Chain::Signet,
      other => bail!("Bitcoin RPC server on unknown chain: {other}"),
    };

    let ord_chain = self.chain();

    if rpc_chain != ord_chain {
      bail!("Bitcoin RPC server is on {rpc_chain} but ord is on {ord_chain}");
    }

    Ok(client)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, bitcoin::Network, std::path::Path};

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
      .cookie_file()
      .unwrap(),
      Path::new("/foo/bar")
    );
  }

  #[test]
  fn use_default_network() {
    let arguments = Arguments::try_parse_from(["ord", "index", "update"]).unwrap();

    assert_eq!(arguments.options.rpc_url(None), "127.0.0.1:8332/");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .ends_with(".cookie"));
  }

  #[test]
  fn uses_network_defaults() {
    let arguments =
      Arguments::try_parse_from(["ord", "--chain=signet", "index", "update"]).unwrap();

    assert_eq!(arguments.options.rpc_url(None), "127.0.0.1:38332/");

    assert!(arguments
      .options
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
  fn rpc_server_chain_must_match() {
    let rpc_server = test_bitcoincore_rpc::builder()
      .network(Network::Testnet)
      .build();

    let options = Options::try_parse_from([
      "ord",
      "--cookie-file",
      rpc_server.cookie_file().to_str().unwrap(),
      "--rpc-url",
      &rpc_server.url(),
    ])
    .unwrap();

    assert_eq!(
      options.bitcoin_rpc_client(None).unwrap_err().to_string(),
      "Bitcoin RPC server is on testnet but ord is on mainnet"
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
        .chain(),
      Chain::Signet
    );
    assert_eq!(
      Arguments::try_parse_from(["ord", "-s", "index", "update"])
        .unwrap()
        .options
        .chain(),
      Chain::Signet
    );

    Arguments::try_parse_from(["ord", "--regtest", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(
      Arguments::try_parse_from(["ord", "--regtest", "index", "update"])
        .unwrap()
        .options
        .chain(),
      Chain::Regtest
    );
    assert_eq!(
      Arguments::try_parse_from(["ord", "-r", "index", "update"])
        .unwrap()
        .options
        .chain(),
      Chain::Regtest
    );

    Arguments::try_parse_from(["ord", "--testnet", "--chain", "signet", "index", "update"])
      .unwrap_err();
    assert_eq!(
      Arguments::try_parse_from(["ord", "--testnet", "index", "update"])
        .unwrap()
        .options
        .chain(),
      Chain::Testnet
    );
    assert_eq!(
      Arguments::try_parse_from(["ord", "-t", "index", "update"])
        .unwrap()
        .options
        .chain(),
      Chain::Testnet
    );
  }

  fn parse_wallet_args(args: &str) -> (Options, subcommand::wallet::Wallet) {
    match Arguments::try_parse_from(args.split_whitespace()) {
      Ok(arguments) => match arguments.subcommand {
        Subcommand::Wallet(wallet) => (arguments.options, wallet),
        subcommand => panic!("unexpected subcommand: {subcommand:?}"),
      },
      Err(err) => panic!("error parsing arguments: {err}"),
    }
  }

  #[test]
  fn wallet_flag_overrides_default_name() {
    let (_, wallet) = parse_wallet_args("ord wallet create");
    assert_eq!(wallet.name, "ord");

    let (_, wallet) = parse_wallet_args("ord wallet --name foo create");
    assert_eq!(wallet.name, "foo")
  }

  #[test]
  fn default_config_is_returned_if_config_option_is_not_passed() {
    assert_eq!(
      Arguments::try_parse_from(["ord", "index", "update"])
        .unwrap()
        .options
        .load_config()
        .unwrap(),
      Default::default()
    );
  }

  #[test]
  fn uses_wallet_rpc() {
    let (options, _) = parse_wallet_args("ord wallet --name foo balance");

    assert_eq!(
      options.rpc_url(Some("foo".into())),
      "127.0.0.1:8332/wallet/foo"
    );
  }

  #[test]
  fn config_is_loaded_from_config_option_path() {
    let id = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path().join("ord.yaml");
    fs::write(&path, format!("hidden:\n- \"{id}\"")).unwrap();

    assert_eq!(
      Arguments::try_parse_from(["ord", "--config", path.to_str().unwrap(), "index", "update"])
        .unwrap()
        .options
        .load_config()
        .unwrap(),
      Config {
        hidden: iter::once(id).collect(),
        ..Default::default()
      }
    );
  }

  #[test]
  fn config_with_rpc_user_pass() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path().join("ord.yaml");
    fs::write(
      &path,
      "hidden:\nbitcoin_rpc_user: foo\nbitcoin_rpc_pass: bar",
    )
    .unwrap();

    assert_eq!(
      Arguments::try_parse_from(["ord", "--config", path.to_str().unwrap(), "index", "update"])
        .unwrap()
        .options
        .load_config()
        .unwrap(),
      Config {
        bitcoin_rpc_user: Some("foo".into()),
        bitcoin_rpc_pass: Some("bar".into()),
        ..Default::default()
      }
    );
  }

  #[test]
  fn config_is_loaded_from_config_dir_option_path() {
    let id = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let tempdir = TempDir::new().unwrap();

    fs::write(
      tempdir.path().join("ord.yaml"),
      format!("hidden:\n- \"{id}\""),
    )
    .unwrap();

    assert_eq!(
      Arguments::try_parse_from([
        "ord",
        "--config-dir",
        tempdir.path().to_str().unwrap(),
        "index",
        "update"
      ])
      .unwrap()
      .options
      .load_config()
      .unwrap(),
      Config {
        hidden: iter::once(id).collect(),
        ..Default::default()
      }
    );
  }

  #[test]
  fn test_derive_var() {
    assert_eq!(Options::derive_var(None, None, None, None).unwrap(), None);

    assert_eq!(
      Options::derive_var(None, None, None, Some("foo")).unwrap(),
      Some("foo".into())
    );

    assert_eq!(
      Options::derive_var(None, None, Some("bar"), Some("foo")).unwrap(),
      Some("bar".into())
    );

    assert_eq!(
      Options::derive_var(Some("qux"), None, Some("bar"), Some("foo")).unwrap(),
      Some("qux".into())
    );

    assert_eq!(
      Options::derive_var(Some("qux"), None, None, Some("foo")).unwrap(),
      Some("qux".into()),
    );
  }

  #[test]
  fn auth_missing_rpc_pass_is_an_error() {
    let options = Options {
      bitcoin_rpc_user: Some("foo".into()),
      ..Default::default()
    };
    assert_eq!(
      options.auth().unwrap_err().to_string(),
      "no bitcoind rpc password specified"
    );
  }

  #[test]
  fn auth_missing_rpc_user_is_an_error() {
    let options = Options {
      bitcoin_rpc_pass: Some("bar".into()),
      ..Default::default()
    };
    assert_eq!(
      options.auth().unwrap_err().to_string(),
      "no bitcoind rpc user specified"
    );
  }

  #[test]
  fn auth_with_user_and_pass() {
    let options = Options {
      bitcoin_rpc_user: Some("foo".into()),
      bitcoin_rpc_pass: Some("bar".into()),
      ..Default::default()
    };
    assert_eq!(
      options.auth().unwrap(),
      Auth::UserPass("foo".into(), "bar".into())
    );
  }

  #[test]
  fn auth_with_cookie_file() {
    let options = Options {
      cookie_file: Some("/var/lib/Bitcoin/.cookie".into()),
      ..Default::default()
    };
    assert_eq!(
      options.auth().unwrap(),
      Auth::CookieFile("/var/lib/Bitcoin/.cookie".into())
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
    .index_runes());
    assert!(
      !Arguments::try_parse_from(["ord", "--index-runes", "index", "update"])
        .unwrap()
        .options
        .index_runes()
    );
    assert!(!Arguments::try_parse_from(["ord", "index", "update"])
      .unwrap()
      .options
      .index_runes());
  }

  #[test]
  fn cookie_file_does_not_exist_error() {
    assert_eq!(
      Options {
        cookie_file: Some("/foo/bar/baz/qux/.cookie".into()),
        ..Default::default()
      }
      .bitcoin_rpc_client(None)
      .map(|_| "")
      .unwrap_err()
      .to_string(),
      "cookie file `/foo/bar/baz/qux/.cookie` does not exist"
    );
  }
}
