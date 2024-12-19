use {
  super::*,
  axum_server::Handle,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  ord::{parse_ord_server_args, Index},
  reqwest::blocking::Response,
  sysinfo::System,
};

pub(crate) struct TestServer {
  bitcoin_rpc_url: String,
  ord_server_handle: Handle,
  port: u16,
  #[allow(unused)]
  tempdir: TempDir,
}

impl TestServer {
  pub(crate) fn spawn(core: &mockcore::Handle) -> Self {
    Self::spawn_with_server_args(core, &[], &[])
  }

  pub(crate) fn spawn_with_args(core: &mockcore::Handle, ord_args: &[&str]) -> Self {
    Self::spawn_with_server_args(core, ord_args, &[])
  }

  pub(crate) fn spawn_with_server_args(
    core: &mockcore::Handle,
    ord_args: &[&str],
    ord_server_args: &[&str],
  ) -> Self {
    let tempdir = TempDir::new().unwrap();

    let cookiefile = tempdir.path().join("cookie");

    fs::write(&cookiefile, "username:password").unwrap();

    let port = TcpListener::bind("127.0.0.1:0")
      .unwrap()
      .local_addr()
      .unwrap()
      .port();

    let (settings, server) = parse_ord_server_args(&format!(
      "ord --bitcoin-rpc-url {} --cookie-file {} --bitcoin-data-dir {} --datadir {} {} server {} --http-port {port} --address 127.0.0.1",
      core.url(),
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

    for i in 0.. {
      match reqwest::blocking::get(format!("http://127.0.0.1:{port}/status")) {
        Ok(_) => break,
        Err(err) => {
          if i == 400 {
            panic!("ord server failed to start: {err}");
          }
        }
      }

      thread::sleep(Duration::from_millis(50));
    }

    Self {
      bitcoin_rpc_url: core.url(),
      ord_server_handle,
      port,
      tempdir,
    }
  }

  pub(crate) fn url(&self) -> Url {
    format!("http://127.0.0.1:{}", self.port).parse().unwrap()
  }

  #[track_caller]
  pub(crate) fn assert_response_regex(&self, path: impl AsRef<str>, regex: impl AsRef<str>) {
    self.sync_server();
    let path = path.as_ref();
    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK, "bad status for {path}: {status}");
    let text = response.text().unwrap();
    assert_regex_match!(text, regex.as_ref());
  }

  #[track_caller]
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

  #[track_caller]
  pub(crate) fn assert_html(
    &self,
    path: impl AsRef<str>,
    chain: Chain,
    content: impl ord::templates::PageContent,
  ) {
    self.sync_server();
    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();

    assert_eq!(
      response.status(),
      StatusCode::OK,
      "{}",
      response.text().unwrap()
    );

    let expected_response = ord::templates::PageHtml::new(
      content,
      Arc::new(ord::subcommand::server::ServerConfig {
        chain,
        domain: Some(System::host_name().unwrap()),
        ..Default::default()
      }),
    )
    .to_string();

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
    let response = reqwest::blocking::get(self.url().join("/update").unwrap()).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().parse::<u64>().unwrap() >= chain_block_count);
  }
}

impl Drop for TestServer {
  fn drop(&mut self) {
    self.ord_server_handle.shutdown();
  }
}
