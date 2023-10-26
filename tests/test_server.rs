use {
  super::*,
  crate::command_builder::ToArgs,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  reqwest::blocking::Response,
};

pub(crate) struct TestServer {
  child: Child,
  port: u16,
  #[allow(unused)]
  tempdir: TempDir,
  rpc_url: String,
}

impl TestServer {
  pub(crate) fn spawn_with_args(
    rpc_server: &test_bitcoincore_rpc::Handle,
    ord_args: &[&str],
  ) -> Self {
    Self::spawn_with_server_args(rpc_server, ord_args, &[])
  }

  pub(crate) fn spawn_with_server_args(
    rpc_server: &test_bitcoincore_rpc::Handle,
    ord_args: &[&str],
    server_args: &[&str],
  ) -> Self {
    let tempdir = TempDir::new().unwrap();
    fs::write(tempdir.path().join(".cookie"), "foo:bar").unwrap();
    let port = TcpListener::bind("127.0.0.1:0")
      .unwrap()
      .local_addr()
      .unwrap()
      .port();

    let child = Command::new(executable_path("ord")).args(format!(
      "--rpc-url {} --bitcoin-data-dir {} --data-dir {} {} server {} --http-port {port} --address 127.0.0.1",
      rpc_server.url(),
      tempdir.path().display(),
      tempdir.path().display(),
      ord_args.join(" "),
      server_args.join(" "),
    ).to_args())
      .env("ORD_INTEGRATION_TEST", "1")
      .current_dir(&tempdir)
      .spawn().unwrap();

    for i in 0.. {
      match reqwest::blocking::get(format!("http://127.0.0.1:{port}/status")) {
        Ok(_) => break,
        Err(err) => {
          if i == 400 {
            panic!("Server failed to start: {err}");
          }
        }
      }

      thread::sleep(Duration::from_millis(25));
    }

    Self {
      child,
      tempdir,
      port,
      rpc_url: rpc_server.url(),
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

  fn sync_server(&self) {
    let client = Client::new(&self.rpc_url, Auth::None).unwrap();
    let chain_block_count = client.get_block_count().unwrap() + 1;

    for i in 0.. {
      let response = reqwest::blocking::get(self.url().join("/blockcount").unwrap()).unwrap();
      assert_eq!(response.status(), StatusCode::OK);
      if response.text().unwrap().parse::<u64>().unwrap() == chain_block_count {
        break;
      } else if i == 20 {
        panic!("index failed to synchronize with chain");
      }
      thread::sleep(Duration::from_millis(25));
    }
  }
}

impl Drop for TestServer {
  fn drop(&mut self) {
    self.child.kill().unwrap()
  }
}
