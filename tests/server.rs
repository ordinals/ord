use super::*;

#[test]
fn run() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let builder = CommandBuilder::new(format!("server --http-port {}", port)).rpc_server(&rpc_server);

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

    thread::sleep(Duration::from_millis(50));
  }

  child.kill().unwrap();
}

#[test]
fn inscription_page() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new(
    "--chain regtest --index-ordinals wallet inscribe --ordinal 5000000000 --file hello.txt",
  )
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_tx = stdout.split("reveal\t").collect::<Vec<&str>>()[1];

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &["--index-ordinals"]);

  ord_server.assert_response_regex(
    &format!("/inscription/{}", reveal_tx),
    ".*<h1>Inscription</h1>
HELLOWORLD.*",
  )
}
