use super::*;

pub(crate) struct TestServer {
  child: Child,
  port: u16,
  #[allow(unused)]
  tempdir: TempDir,
}

impl TestServer {
  pub(crate) fn spawn(rpc_server: &test_bitcoincore_rpc::Handle) -> Self {
    let tempdir = TempDir::new().unwrap();
    fs::create_dir(tempdir.path().join("regtest")).unwrap();
    fs::write(tempdir.path().join("regtest/.cookie"), "foo:bar").unwrap();
    let port = TcpListener::bind("127.0.0.1:0")
      .unwrap()
      .local_addr()
      .unwrap()
      .port();
    let child = CommandBuilder::new(format!(
      "--chain regtest --rpc-url {} --bitcoin-data-dir {} --data-dir {} server --http-port {port} --address 127.0.0.1",
      rpc_server.url(),
      tempdir.path().display(),
      tempdir.path().display()
    ))
    .command()
    .spawn()
    .unwrap();

    for i in 0.. {
      match reqwest::blocking::get(&format!("http://127.0.0.1:{port}/status")) {
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
    }
  }

  pub(crate) fn url(&self) -> Url {
    format!("http://127.0.0.1:{}", self.port).parse().unwrap()
  }

  pub(crate) fn assert_response_regex(&self, path: &str, status: StatusCode, regex: &str) {
    let response = reqwest::blocking::get(self.url().join(path).unwrap()).unwrap();
    assert_eq!(response.status(), status);
    assert_regex_match!(response.text().unwrap(), regex);
  }
}

impl Drop for TestServer {
  fn drop(&mut self) {
    self.child.kill().unwrap()
  }
}
