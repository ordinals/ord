use super::*;

#[test]
#[ignore]
fn run() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let builder = CommandBuilder::new(format!("preview --http-port {port} foo.txt"))
    .rpc_server(&rpc_server)
    .write("foo.txt", "TEST_INSCRIPTION");

  let mut command = builder.command();

  let mut child = command.spawn().unwrap();

  for attempt in 0.. {
    if let Ok(response) = reqwest::blocking::get(format!("http://localhost:{port}/status")) {
      if response.status() == 200 {
        assert_eq!(response.text().unwrap(), "OK");
        break;
      }
    }

    if attempt == 100 {
      panic!("Server did not respond to status check",);
    }

    thread::sleep(Duration::from_millis(500));
  }

  assert!(
    reqwest::blocking::get(format!("http://localhost:{port}/inscriptions"))
      .unwrap()
      .text()
      .unwrap()
      .contains("TEST_INSCRIPTION")
  );

  child.kill().unwrap();
}
