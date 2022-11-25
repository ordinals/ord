use {
  super::*,
  bitcoincore_rpc::{Auth, Client},
};

#[derive(Debug, Parser)]
pub(crate) struct Options {
  #[clap(long, help = "Load Bitcoin Core data dir from <BITCOIN_DATA_DIR>.")]
  bitcoin_data_dir: Option<PathBuf>,
  #[clap(long, arg_enum, default_value = "mainnet", help = "Index <CHAIN>.")]
  pub(crate) chain: Chain,
  #[clap(long, help = "Load Bitcoin Core RPC cookie file from <COOKIE_FILE>.")]
  cookie_file: Option<PathBuf>,
  #[clap(long, help = "Store index in <DATA_DIR>.")]
  data_dir: Option<PathBuf>,
  #[clap(long, help = "Limit index to <HEIGHT_LIMIT> blocks.")]
  pub(crate) height_limit: Option<u64>,
  #[clap(long, help = "Use index at <INDEX>.")]
  pub(crate) index: Option<PathBuf>,
  #[clap(long, help = "Index ordinal ranges.")]
  pub(crate) index_ordinals: bool,
  #[clap(long, help = "Connect to Bitcoin Core RPC at <RPC_URL>.")]
  rpc_url: Option<String>,
}

impl Options {
  pub(crate) fn rpc_url(&self) -> String {
    self
      .rpc_url
      .as_ref()
      .unwrap_or(&format!("127.0.0.1:{}", self.chain.default_rpc_port(),))
      .into()
  }

  pub(crate) fn cookie_file(&self) -> Result<PathBuf> {
    if let Some(cookie_file) = &self.cookie_file {
      return Ok(cookie_file.clone());
    }

    let path = if let Some(bitcoin_data_dir) = &self.bitcoin_data_dir {
      bitcoin_data_dir.clone()
    } else if cfg!(target_os = "linux") {
      dirs::home_dir()
        .ok_or_else(|| anyhow!("failed to retrieve home dir"))?
        .join(".bitcoin")
    } else {
      dirs::data_dir()
        .ok_or_else(|| anyhow!("failed to retrieve data dir"))?
        .join("Bitcoin")
    };

    let path = self.chain.join_with_data_dir(&path);

    Ok(path.join(".cookie"))
  }

  pub(crate) fn data_dir(&self) -> Result<PathBuf> {
    let base = match &self.data_dir {
      Some(base) => base.clone(),
      None => dirs::data_dir()
        .ok_or_else(|| anyhow!("failed to retrieve data dir"))?
        .join("ord"),
    };

    Ok(self.chain.join_with_data_dir(&base))
  }

  pub(crate) fn bitcoin_rpc_client(&self) -> Result<Client> {
    let cookie_file = self.cookie_file()?;
    let rpc_url = self.rpc_url();
    log::info!(
      "Connecting to Bitcoin Core RPC server at {rpc_url} using credentials from `{}`",
      cookie_file.display()
    );

    let client = Client::new(&rpc_url, Auth::CookieFile(cookie_file))
      .with_context(|| format!("failed to connect to Bitcoin Core RPC at {rpc_url}"))?;

    let rpc_chain = match client.get_blockchain_info()?.chain.as_str() {
      "main" => Chain::Mainnet,
      "test" => Chain::Testnet,
      "regtest" => Chain::Regtest,
      "signet" => Chain::Signet,
      other => bail!("Bitcoin RPC server on unknown chain: {other}"),
    };

    let ord_chain = self.chain;

    if rpc_chain != ord_chain {
      bail!("Bitcoin RPC server is on {rpc_chain} but ord is on {ord_chain}");
    }

    Ok(client)
  }

  pub(crate) fn bitcoin_rpc_client_mainnet_forbidden(&self, command: &str) -> Result<Client> {
    let client = self.bitcoin_rpc_client()?;

    if self.chain == Chain::Mainnet {
      bail!("`{command}` is unstable and not yet supported on mainnet.");
    }
    Ok(client)
  }

  pub(crate) fn bitcoin_rpc_client_for_wallet_command(&self, command: &str) -> Result<Client> {
    let client = self.bitcoin_rpc_client()?;

    if self.chain == Chain::Mainnet {
      let wallet_info = client.get_wallet_info()?;

      if !(wallet_info.wallet_name == "ord" || wallet_info.wallet_name.starts_with("ord-")) {
        bail!("`{command}` may only be used on mainnet with a wallet named `ord` or whose name starts with `ord-`");
      }

      let balances = client.get_balances()?;

      let total = balances.mine.trusted + balances.mine.untrusted_pending + balances.mine.immature;

      if total > Amount::from_sat(1_000_000) {
        bail!(
          "`{command}` may not be used on mainnet with wallets containing more than 1,000,000 sats"
        );
      }
    }
    Ok(client)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::path::Path};

  #[test]
  fn rpc_url_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from(["ord", "--rpc-url=127.0.0.1:1234", "--chain=signet", "index"])
        .unwrap()
        .options
        .rpc_url(),
      "127.0.0.1:1234"
    );
  }

  #[test]
  fn cookie_file_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from(["ord", "--cookie-file=/foo/bar", "--chain=signet", "index"])
        .unwrap()
        .options
        .cookie_file()
        .unwrap(),
      Path::new("/foo/bar")
    );
  }

  #[test]
  fn use_default_network() {
    let arguments = Arguments::try_parse_from(["ord", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:8332");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .ends_with(".cookie"));
  }

  #[test]
  fn uses_network_defaults() {
    let arguments = Arguments::try_parse_from(["ord", "--chain=signet", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:38332");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string()
      .ends_with("/signet/.cookie"))
  }

  #[test]
  fn mainnet_cookie_file_path() {
    let cookie_file = Arguments::try_parse_from(["ord", "index"])
      .unwrap()
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    if cfg!(target_os = "linux") {
      assert!(cookie_file.ends_with("/.bitcoin/.cookie"));
    } else {
      assert!(cookie_file.ends_with("/Bitcoin/.cookie"));
    }
  }

  #[test]
  fn othernet_cookie_file_path() {
    let arguments = Arguments::try_parse_from(["ord", "--chain=signet", "index"]).unwrap();

    let cookie_file = arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    if cfg!(target_os = "linux") {
      assert!(cookie_file.ends_with("/.bitcoin/signet/.cookie"));
    } else {
      assert!(cookie_file.ends_with("/Bitcoin/signet/.cookie"));
    }
  }

  #[test]
  fn cookie_file_defaults_to_bitcoin_data_dir() {
    let arguments =
      Arguments::try_parse_from(["ord", "--bitcoin-data-dir=foo", "--chain=signet", "index"])
        .unwrap();

    let cookie_file = arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    assert!(cookie_file.ends_with("foo/signet/.cookie"));
  }

  #[test]
  fn mainnet_data_dir() {
    let data_dir = Arguments::try_parse_from(["ord", "index"])
      .unwrap()
      .options
      .data_dir()
      .unwrap()
      .display()
      .to_string();
    assert!(data_dir.ends_with("/ord"), "{data_dir}");
  }

  #[test]
  fn othernet_data_dir() {
    let data_dir = Arguments::try_parse_from(["ord", "--chain=signet", "index"])
      .unwrap()
      .options
      .data_dir()
      .unwrap()
      .display()
      .to_string();
    assert!(data_dir.ends_with("/ord/signet"), "{data_dir}");
  }

  #[test]
  fn network_is_joined_with_data_dir() {
    let data_dir =
      Arguments::try_parse_from(["ord", "--chain=signet", "--data-dir", "foo", "index"])
        .unwrap()
        .options
        .data_dir()
        .unwrap()
        .display()
        .to_string();
    assert!(data_dir.ends_with("foo/signet"), "{data_dir}");
  }

  #[test]
  fn network_accepts_aliases() {
    fn check_network_alias(alias: &str, suffix: &str) {
      let data_dir = Arguments::try_parse_from(["ord", "--chain", alias, "index"])
        .unwrap()
        .options
        .data_dir()
        .unwrap()
        .display()
        .to_string();

      assert!(data_dir.ends_with(suffix), "{data_dir}");
    }

    check_network_alias("main", "ord");
    check_network_alias("mainnet", "ord");
    check_network_alias("regtest", "ord/regtest");
    check_network_alias("signet", "ord/signet");
    check_network_alias("test", "ord/testnet3");
    check_network_alias("testnet", "ord/testnet3");
  }

  #[test]
  fn rpc_server_chain_must_match() {
    let rpc_server = test_bitcoincore_rpc::spawn_with(bitcoin::Network::Testnet, "ord");

    let tempdir = TempDir::new().unwrap();

    let cookie_file = tempdir.path().join(".cookie");
    fs::write(&cookie_file, "username:password").unwrap();

    let options = Options::try_parse_from([
      "ord",
      "--cookie-file",
      cookie_file.to_str().unwrap(),
      "--rpc-url",
      &rpc_server.url(),
    ])
    .unwrap();

    assert_eq!(
      options.bitcoin_rpc_client().unwrap_err().to_string(),
      "Bitcoin RPC server is on testnet but ord is on mainnet"
    );
  }
}
