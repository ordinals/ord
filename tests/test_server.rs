use {
  super::*,
  axum_server::Handle,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  ord::{parse_ord_server_args, Index},
  reqwest::blocking::Response,
};

pub(crate) struct TestServer {
  bitcoin_rpc_url: String,
  ord_server_handle: Handle,
  port: u16,
  #[allow(unused)]
  tempdir: TempDir,
}

impl TestServer {
  pub(crate) fn spawn(bitcoin_rpc_server: &test_bitcoincore_rpc::Handle) -> Self {
    Self::spawn_with_server_args(bitcoin_rpc_server, &[], &[])
  }

  pub(crate) fn spawn_with_args(
    bitcoin_rpc_server: &test_bitcoincore_rpc::Handle,
    ord_args: &[&str],
  ) -> Self {
    Self::spawn_with_server_args(bitcoin_rpc_server, ord_args, &[])
  }

  pub(crate) fn spawn_with_server_args(
    bitcoin_rpc_server: &test_bitcoincore_rpc::Handle,
    ord_args: &[&str],
    ord_server_args: &[&str],
  ) -> Self {
    std::env::set_var("ORD_INTEGRATION_TEST", "1");

    let tempdir = TempDir::new().unwrap();

    let cookiefile = tempdir.path().join("cookie");

    fs::write(&cookiefile, "username:password").unwrap();

    let port = TcpListener::bind("127.0.0.1:0")
      .unwrap()
      .local_addr()
      .unwrap()
      .port();

    let (settings, server) = parse_ord_server_args(&format!(
      "ord --rpc-url {} --cookie-file {} --bitcoin-data-dir {} --data-dir {} {} server {} --http-port {port} --address 127.0.0.1",
      bitcoin_rpc_server.url(),
      cookiefile.to_str().unwrap(),
      tempdir.path().display(),
      tempdir.path().display(),
      ord_args.join(" "),
      ord_server_args.join(" "),
    ));

    let index = Arc::new(Index::open(&settings).unwrap());
    let ord_server_handle = Handle::new();

    {
      let index = index.clone();
      let ord_server_handle = ord_server_handle.clone();
      thread::spawn(|| server.run(settings, index, ord_server_handle).unwrap());
    }

    let mut attempts = 0;
    let max_attempts = 500;
    let mut backoff = 50;
    while attempts < max_attempts {
      match reqwest::blocking::get(format!("http://127.0.0.1:{port}/status")) {
        Ok(_) => break,
        Err(err) if attempts == max_attempts - 1 => {
          panic!("mock ord server failed to start: {err}")
        }
        Err(_) => {
          thread::sleep(Duration::from_millis(backoff));
          backoff = (backoff as f64 * 1.5) as u64;
        }
      }
      attempts += 1;
    }

    Self {
      bitcoin_rpc_url: bitcoin_rpc_server.url(),
      ord_server_handle,
      port,
      tempdir,
    }
  }

  pub(crate) fn url(&self) -> Url {
    format!("http://127.0.0.1:{}", self.port).parse().unwrap()
  }

  pub(crate) fn assert_response_regex(&self, path: impl AsRef<str>, regex: impl AsRef<str>) {
    self.sync_server();

    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_regex_match!(response.text().unwrap(), regex.as_ref());
  }

  pub(crate) fn assert_response(&self, path: impl AsRef<str>, expected_response: &str) {
    self.sync_server();
    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();
    assert_eq!(
      response.status(),
      StatusCode::OK,
      "{}",
      response.text().unwrap()
    );
    pretty_assert_eq!(response.text().unwrap(), expected_response);
  }

  pub(crate) fn request(&self, path: impl AsRef<str>) -> Response {
    self.sync_server();

    reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap()
  }

  pub(crate) fn json_request(&self, path: impl AsRef<str>) -> Response {
    self.sync_server();

    let client = reqwest::blocking::Client::new();

    client
      .get(self.url().join(path.as_ref()).unwrap())
      .header(reqwest::header::ACCEPT, "application/json")
      .send()
      .unwrap()
  }

  pub(crate) fn sync_server(&self) {
    let client = Client::new(&self.bitcoin_rpc_url, Auth::None).unwrap();
    let chain_block_count = client.get_block_count().unwrap() + 1;
    let max_attempts = 100;
    let mut backoff = 50;

    for _ in 0..max_attempts {
      let response = match reqwest::blocking::get(self.url().join("/blockcount").unwrap()) {
        Ok(res) => res,
        Err(_) => {
          thread::sleep(Duration::from_millis(backoff));
          continue;
        }
      };

      if response.status() == StatusCode::OK {
        let ord_height = response.text().unwrap().parse::<u64>().unwrap();
        if ord_height >= chain_block_count {
          return;
        }
      }
      thread::sleep(Duration::from_millis(backoff));
      backoff = (backoff as f64 * 1.5) as u64;
    }
    panic!(
      "Index failed to synchronize with chain after {} attempts",
      max_attempts
    );
  }
}

impl Drop for TestServer {
  fn drop(&mut self) {
    self.ord_server_handle.shutdown();
  }
}
