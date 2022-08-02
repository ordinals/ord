use super::*;

pub(crate) struct State {
  bitcoind: Child,
  pub(crate) tempdir: TempDir,
  pub(crate) client: Client,
  pub(crate) wallet: Wallet<MemoryDatabase>,
  pub(crate) blockchain: RpcBlockchain,
  pub(crate) bitcoind_rpc_port: u16,
  // server: Child,
}

impl State {
  pub(crate) fn new() -> Self {
    let tempdir = TempDir::new().unwrap();

    fs::create_dir(tempdir.path().join("bitcoin")).unwrap();

    let rpc_port = free_port();

    let bitcoind = Command::new("bitcoind")
      .stdout(if log::max_level() >= LevelFilter::Info {
        Stdio::inherit()
      } else {
        Stdio::piped()
      })
      .args(&[
        "-minrelaytxfee=0",
        "-blockmintxfee=0",
        "-dustrelayfee=0",
        "-maxtxfee=21000000",
        "-datadir=bitcoin",
        "-regtest",
        "-networkactive=0",
        "-listen=0",
        &format!("-rpcport={rpc_port}"),
      ])
      .current_dir(&tempdir.path())
      .spawn()
      .unwrap();

    let cookiefile = tempdir.path().join("bitcoin/regtest/.cookie");

    while !cookiefile.is_file() {}

    let client = Client::new(
      &format!("localhost:{rpc_port}"),
      bitcoincore_rpc::Auth::CookieFile(cookiefile.clone()),
    )
    .unwrap();

    for attempt in 0..=300 {
      match client.get_blockchain_info() {
        Ok(_) => break,
        Err(err) => {
          if attempt == 300 {
            panic!("Failed to connect to bitcoind: {err}");
          }
        }
      }
      sleep(Duration::from_millis(100));
    }

    // let server = Command::new(executable_path("ord"))
    //   .envs(self.envs.clone())
    //   .current_dir(&tempdir)
    //   .arg(format!("--rpc-url=localhost:{}", rpc_port))
    //   .arg("--cookie-file=bitcoin/regtest/.cookie")
    //   .args(["ord", "server", "--address", "127.0.0.1", "--http-port"])
    //   .spawn()
    //   .unwrap();

    let wallet = Wallet::new(
      Bip84(
        (
          Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")
            .unwrap(),
          None,
        ),
        KeychainKind::External,
      ),
      None,
      Network::Regtest,
      MemoryDatabase::new(),
    )
    .unwrap();

    let blockchain = RpcBlockchain::from_config(&RpcConfig {
      url: format!("localhost:{rpc_port}"),
      auth: bdk::blockchain::rpc::Auth::Cookie { file: cookiefile },
      network: Network::Regtest,
      wallet_name: "test".to_string(),
      skip_blocks: None,
    })
    .unwrap();

    State {
      tempdir,
      rpc_port,
      bitcoind,
      client,
      wallet,
      blockchain,
    }
  }

  pub(crate) fn request(mut self, path: &str, status: u16, expected_response: &str) -> Self {
    let response =
      reqwest::blocking::get(&format!("http://127.0.0.1:{}/{path}", self.ord_http_port)).unwrap();
    log::info!("{:?}", response);
    assert_eq!(response.status().as_u16(), status);
    assert_eq!(response.text().unwrap(), expected_response);
    self
  }
}

impl Drop for State {
  fn drop(&mut self) {
    self.bitcoind.kill().unwrap();
  }
}
