use {super::*, bitcoincore_rpc::Auth};

#[derive(Clone, Default, Debug, Parser)]
#[command(group(
  ArgGroup::new("chains")
    .required(false)
    .args(&["chain_argument", "signet", "regtest", "testnet"]),
))]
pub struct Options {
  #[arg(long, help = "Minify JSON output.")]
  pub(crate) minify: bool,
  #[arg(long, help = "Load Bitcoin Core data dir from <BITCOIN_DATA_DIR>.")]
  pub(crate) bitcoin_data_dir: Option<PathBuf>,
  #[arg(long, help = "Authenticate to Bitcoin Core RPC with <RPC_PASS>.")]
  pub(crate) bitcoin_rpc_pass: Option<String>,
  #[arg(long, help = "Authenticate to Bitcoin Core RPC as <RPC_USER>.")]
  pub(crate) bitcoin_rpc_user: Option<String>,
  #[arg(long = "chain", value_enum, help = "Use <CHAIN>. [default: maiinnet]")]
  pub(crate) chain_argument: Option<Chain>,
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
  #[arg(long, help = "Keep sat index entries of spent outputs.")]
  pub(crate) index_spent_sats: bool,
  #[arg(long, help = "Store transactions in index.")]
  pub(crate) index_transactions: bool,
  #[arg(
    long,
    short,
    alias = "noindex_inscriptions",
    help = "Do not index inscriptions."
  )]
  pub(crate) no_index_inscriptions: bool,
  #[arg(
    long,
    requires = "username",
    help = "Require basic HTTP authentication with <PASSWORD>. Credentials are sent in cleartext. Consider using authentication in conjunction with HTTPS."
  )]
  pub(crate) password: Option<String>,
  #[arg(long, short, help = "Use regtest. Equivalent to `--chain regtest`.")]
  pub(crate) regtest: bool,
  #[arg(long, help = "Connect to Bitcoin Core RPC at <RPC_URL>.")]
  pub(crate) rpc_url: Option<String>,
  #[arg(long, short, help = "Use signet. Equivalent to `--chain signet`.")]
  pub(crate) signet: bool,
  #[arg(long, short, help = "Use testnet. Equivalent to `--chain testnet`.")]
  pub(crate) testnet: bool,
  #[arg(
    long,
    requires = "password",
    help = "Require basic HTTP authentication with <USERNAME>. Credentials are sent in cleartext. Consider using authentication in conjunction with HTTPS."
  )]
  pub(crate) username: Option<String>,
}

impl Options {
  fn default_data_dir() -> PathBuf {
    dirs::data_dir()
      .map(|dir| dir.join("ord"))
      .expect("failed to retrieve data dir")
  }

  #[cfg(test)]
  pub(crate) fn settings(self) -> Result<Settings> {
    Settings::new(self)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::path::Path, tempfile::TempDir};

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
        .settings()
        .unwrap()
        .hidden,
      iter::once(id).collect(),
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
        .settings()
        .unwrap()
        .auth()
        .unwrap(),
      Auth::UserPass("foo".into(), "bar".into()),
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
      .settings()
      .unwrap()
      .hidden,
      iter::once(id).collect(),
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
}
