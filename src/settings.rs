use {super::*, bitcoincore_rpc::Auth};

#[derive(Default, Debug, Clone)]
pub struct Settings {
  pub(crate) chain: Chain,
  pub(crate) config: Config,
  pub(crate) options: Options,
}

impl Settings {
  pub(crate) fn new(options: Options) -> Result<Self> {
    let config: Config = match &options.config {
      Some(path) => serde_yaml::from_reader(File::open(path)?)?,
      None => match &options.config_dir {
        Some(dir) if dir.join("ord.yaml").exists() => {
          serde_yaml::from_reader(File::open(dir.join("ord.yaml"))?)?
        }
        Some(_) | None => Default::default(),
      },
    };

    let chain = Self::setting_typed(
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

    Ok(Self {
      config,
      options,
      chain,
    })
  }

  pub(crate) fn auth(&self) -> Result<Auth> {
    let rpc_user = Self::setting(
      self.options.bitcoin_rpc_user.as_deref(),
      Some("BITCOIN_RPC_USER"),
      self.config.bitcoin_rpc_user.as_deref(),
      None,
    )?;

    let rpc_pass = Self::setting(
      self.options.bitcoin_rpc_pass.as_deref(),
      Some("BITCOIN_RPC_PASS"),
      self.config.bitcoin_rpc_pass.as_deref(),
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
    if let Some(cookie_file) = &self.options.cookie_file {
      return Ok(cookie_file.clone());
    }

    let path = if let Some(bitcoin_data_dir) = &self.options.bitcoin_data_dir {
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
      .options
      .username
      .as_deref()
      .zip(self.options.password.as_deref())
  }

  pub(crate) fn data_dir(&self) -> PathBuf {
    self.chain().join_with_data_dir(&self.options.data_dir)
  }

  pub(crate) fn first_inscription_height(&self) -> u32 {
    if integration_test() {
      0
    } else {
      self
        .options
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
    self.options.index_runes && self.chain() != Chain::Mainnet
  }

  pub(crate) fn rpc_url(&self, wallet_name: Option<String>) -> String {
    let base_url = self
      .options
      .rpc_url
      .clone()
      .unwrap_or(format!("127.0.0.1:{}", self.chain().default_rpc_port()));

    match wallet_name {
      Some(wallet_name) => format!("{base_url}/wallet/{wallet_name}"),
      None => format!("{base_url}/"),
    }
  }

  fn setting_typed<T: FromStr<Err = Error>>(
    arg_value: Option<T>,
    env_key: Option<&str>,
    config_value: Option<T>,
    default_value: T,
  ) -> Result<T> {
    if let Some(arg_value) = arg_value {
      return Ok(arg_value);
    }

    if let Some(env_key) = env_key {
      let key = format!("ORD_{env_key}");
      match env::var(key) {
        Ok(env_value) => {
          return env_value
            .parse()
            .with_context(|| anyhow!("failed to parse {env_key}"))
        }
        Err(err @ env::VarError::NotUnicode(_)) => return Err(err.into()),
        Err(env::VarError::NotPresent) => {}
      }
    }

    if let Some(config_value) = config_value {
      return Ok(config_value);
    }

    Ok(default_value)
  }

  fn setting(
    arg_value: Option<&str>,
    env_key: Option<&str>,
    config_value: Option<&str>,
    default_value: Option<&str>,
  ) -> Result<Option<String>> {
    if let Some(arg_value) = arg_value {
      return Ok(Some(arg_value.into()));
    }

    if let Some(env_key) = env_key {
      match env::var(format!("ORD_{env_key}")) {
        Ok(env_value) => return Ok(Some(env_value)),
        Err(err @ env::VarError::NotUnicode(_)) => return Err(err.into()),
        Err(env::VarError::NotPresent) => {}
      }
    }

    Ok(config_value.or(default_value).map(str::to_string))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn settings(args: &[&str]) -> Settings {
    let options = Options::try_parse_from(args).unwrap();

    Settings {
      options,
      ..Default::default()
    }
  }

  #[test]
  fn auth_missing_rpc_pass_is_an_error() {
    let settings = Settings {
      options: Options {
        bitcoin_rpc_user: Some("foo".into()),
        ..Default::default()
      },
      ..Default::default()
    };

    assert_eq!(
      settings.auth().unwrap_err().to_string(),
      "no bitcoind rpc password specified"
    );
  }

  #[test]
  fn auth_missing_rpc_user_is_an_error() {
    let settings = Settings {
      options: Options {
        bitcoin_rpc_pass: Some("bar".into()),
        ..Default::default()
      },
      ..Default::default()
    };
    assert_eq!(
      settings.auth().unwrap_err().to_string(),
      "no bitcoind rpc user specified"
    );
  }

  #[test]
  fn auth_with_user_and_pass() {
    let settings = Settings {
      options: Options {
        bitcoin_rpc_user: Some("foo".into()),
        bitcoin_rpc_pass: Some("bar".into()),
        ..Default::default()
      },
      ..Default::default()
    };
    assert_eq!(
      settings.auth().unwrap(),
      Auth::UserPass("foo".into(), "bar".into())
    );
  }

  #[test]
  fn auth_with_cookie_file() {
    let settings = Settings {
      options: Options {
        cookie_file: Some("/var/lib/Bitcoin/.cookie".into()),
        ..Default::default()
      },
      ..Default::default()
    };
    assert_eq!(
      settings.auth().unwrap(),
      Auth::CookieFile("/var/lib/Bitcoin/.cookie".into())
    );
  }

  #[test]
  fn cookie_file_does_not_exist_error() {
    assert_eq!(
      Settings {
        options: Options {
          cookie_file: Some("/foo/bar/baz/qux/.cookie".into()),
          ..Default::default()
        },
        ..Default::default()
      }
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
    assert_eq!(Settings::setting(None, None, None, None).unwrap(), None);

    assert_eq!(
      Settings::setting(None, None, None, Some("foo")).unwrap(),
      Some("foo".into())
    );

    assert_eq!(
      Settings::setting(None, None, Some("bar"), Some("foo")).unwrap(),
      Some("bar".into())
    );

    assert_eq!(
      Settings::setting(Some("qux"), None, Some("bar"), Some("foo")).unwrap(),
      Some("qux".into())
    );

    assert_eq!(
      Settings::setting(Some("qux"), None, None, Some("foo")).unwrap(),
      Some("qux".into()),
    );
  }
}
