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
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_tx = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    &format!("/inscription/{reveal_tx}"),
    &format!(
      ".*<meta property=og:image content='/content/{reveal_tx}'>.*
<h1>Inscription {reveal_tx}</h1>
<a class=content href=/content/{reveal_tx}>
<pre>HELLOWORLD</pre>
</a>
<dl>
  <dt>content size</dt>
  <dd>10 bytes</dd>
  <dt>content type</dt>
  <dd>text/plain;charset=utf-8</dd>
  <dt>genesis height</dt>
  <dd>2</dd>
  <dt>genesis transaction</dt>
  <dd><a class=monospace href=/tx/{reveal_tx}>{reveal_tx}</a></dd>
  <dt>location</dt>
  <dd class=monospace>{reveal_tx}:0:0</dd>
</dl>.*",
    ),
  );
}

#[test]
fn inscription_appears_on_reveal_transaction_page() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_tx = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    &format!("/tx/{}", reveal_tx),
    ".*<h1>Transaction .*</h1>.*HELLOWORLD.*",
  );
}

#[test]
fn inscription_page_after_send() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    &format!("/inscription/{reveal_txid}"),
    &format!(
      ".*<h1>Inscription {reveal_txid}</h1>.*HELLOWORLD.*<dl>.*<dt>location</dt>
  <dd class=monospace>{reveal_txid}:0:0</dd>
</dl>.*",
    ),
  );

  let txid = CommandBuilder::new(format!(
    "--chain regtest wallet send {reveal_txid} bcrt1q6rhpng9evdsfnn833a4f4vej0asu6dk5srld6x"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex(".*")
  .run();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    &format!("/inscription/{reveal_txid}"),
    &format!(
      ".*<h1>Inscription {reveal_txid}</h1>.*HELLOWORLD.*
<dl>
  .*
  <dt>location</dt>
  <dd class=monospace>{}:0:0</dd>
</dl>.*",
      txid.trim(),
    ),
  )
}

#[test]
fn inscription_content() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_tx = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).request(&format!("/content/{reveal_tx}"));

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(response.bytes().unwrap(), "HELLOWORLD");
}

#[test]
fn home_page_includes_latest_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let inscription_id = create_inscription(&rpc_server, "foo.png");

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/",
    &format!(
      ".*<h2>Latest Inscriptions</h2>
<div class=inscriptions>
  <a href=/inscription/{inscription_id}><img .*></a>
</div>.*"
    ),
  );
}

#[test]
fn home_page_only_includes_graphical_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  create_inscription(&rpc_server, "hello.txt");
  let inscription_id = create_inscription(&rpc_server, "foo.png");

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/",
    &format!(
      ".*<h2>Latest Inscriptions</h2>
<div class=inscriptions>
  <a href=/inscription/{inscription_id}><img .*></a>
</div>.*"
    ),
  );
}

#[test]
fn home_page_inscriptions_are_sorted() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let mut inscriptions = String::new();

  for i in 0..8 {
    let id = create_inscription(&rpc_server, &format!("{i}.png"));
    inscriptions.insert_str(0, &format!("\n  <a href=/inscription/{id}><img .*></a>"));
  }

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/",
    &format!(
      ".*<h2>Latest Inscriptions</h2>
<div class=inscriptions>{inscriptions}
</div>.*"
    ),
  );
}

#[test]
fn inscriptions_page() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_tx = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/inscriptions",
    &format!(
      ".*<h1>Inscriptions</h1>.*
<ul>
  <li>
    <a href=/inscription/{reveal_tx} class=monospace>
      {reveal_tx}
    </a>
  </li>
</ul>.*",
    ),
  );
}

#[test]
fn inscriptions_page_is_sorted() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let mut inscriptions = String::new();

  for i in 0..8 {
    let id = create_inscription(&rpc_server, &format!("{i}.png"));
    inscriptions.insert_str(
      0,
      &format!(".*<a href=/inscription/{id} class=monospace>.*"),
    );
  }

  TestServer::spawn_with_args(&rpc_server, &[])
    .assert_response_regex("/inscriptions", &inscriptions);
}
