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
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe {
    inscription,
    reveal,
    ..
  } = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      ".*<meta property=og:image content='/content/{inscription}'>.*
<h1>Inscription 0</h1>
.*<a href=/preview/{inscription}><iframe .* src=/preview/{inscription}></iframe></a>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{inscription}</dd>
  <dt>address</dt>
  <dd class=monospace>bc1.*</dd>
  <dt>output value</dt>
  <dd>9862</dd>
  <dt>content</dt>
  <dd><a href=/content/{inscription}>link</a></dd>
  <dt>content size</dt>
  <dd>3 bytes</dd>
  <dt>content type</dt>
  <dd>text/plain;charset=utf-8</dd>
  <dt>timestamp</dt>
  <dd>1970-01-01 00:00:02</dd>
  <dt>genesis height</dt>
  <dd>2</dd>
  <dt>genesis transaction</dt>
  <dd><a class=monospace href=/tx/{reveal}>{reveal}</a></dd>
  <dt>location</dt>
  <dd class=monospace>{reveal}:0:0</dd>
  <dt>output</dt>
  <dd><a class=monospace href=/output/{reveal}:0>{reveal}:0</a></dd>
  <dt>offset</dt>
  <dd>0</dd>
</dl>.*",
    ),
  );
}

#[test]
fn inscription_appears_on_reveal_transaction_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe { reveal, .. } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    format!("/tx/{reveal}"),
    format!(".*<h1>Transaction .*</h1>.*<a href=/inscription/{reveal}.*"),
  );
}

#[test]
fn inscription_appears_on_output_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe {
    reveal,
    inscription,
    ..
  } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    format!("/output/{reveal}:0"),
    format!(".*<h1>Output <span class=monospace>{reveal}:0</span></h1>.*<a href=/inscription/{inscription}.*"),
  );
}

#[test]
fn inscription_page_after_send() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe {
    reveal,
    inscription,
    ..
  } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      r".*<h1>Inscription 0</h1>.*<dt>location</dt>\s*<dd class=monospace>{reveal}:0:0</dd>.*",
    ),
  );

  let txid = CommandBuilder::new(format!(
    "wallet send bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {inscription}"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(".*")
  .run();

  rpc_server.mine_blocks(1);

  let send = txid.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      r".*<h1>Inscription 0</h1>.*<dt>address</dt>\s*<dd class=monospace>bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv</dd>.*<dt>location</dt>\s*<dd class=monospace>{send}:0:0</dd>.*",
    ),
  )
}

#[test]
fn inscription_content() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let response = TestServer::spawn_with_args(&rpc_server, &[])
    .request(&format!("/content/{}", output.inscription));

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(
    response.headers().get("content-security-policy").unwrap(),
    "default-src 'unsafe-eval' 'unsafe-inline'"
  );
  assert_eq!(response.bytes().unwrap(), "FOO");
}

#[test]
fn home_page_includes_latest_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let output = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/",
    format!(
      ".*<h2>Latest Inscriptions</h2>
<div class=thumbnails>
  <a href=/inscription/{}><iframe .*></a>
</div>.*",
      output.inscription
    ),
  );
}

#[test]
fn home_page_inscriptions_are_sorted() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let mut inscriptions = String::new();

  for _ in 0..8 {
    let output = inscribe(&rpc_server);
    inscriptions.insert_str(
      0,
      &format!(
        "\n  <a href=/inscription/{}><iframe .*></a>",
        output.inscription
      ),
    );
  }

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/",
    format!(
      ".*<h2>Latest Inscriptions</h2>
<div class=thumbnails>{inscriptions}
</div>.*"
    ),
  );
}

#[test]
fn inscriptions_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let output = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/inscriptions",
    format!(
      ".*<h1>Inscriptions</h1>
<div class=thumbnails>
  <a href=/inscription/{}>.*</a>
</div>
.*",
      output.inscription,
    ),
  );
}

#[test]
fn inscriptions_page_is_sorted() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let mut inscriptions = String::new();

  for _ in 0..8 {
    inscriptions.insert_str(
      0,
      &format!(
        ".*<a href=/inscription/{}>.*",
        inscribe(&rpc_server).inscription
      ),
    );
  }

  TestServer::spawn_with_args(&rpc_server, &[])
    .assert_response_regex("/inscriptions", &inscriptions);
}

#[test]
fn inscriptions_page_has_next_and_previous() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let a = inscribe(&rpc_server);
  let b = inscribe(&rpc_server);
  let c = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    format!("/inscription/{}", b.inscription),
    format!(
      ".*<h1>Inscription 1</h1>.*
<div class=inscription>
<a class=previous href=/inscription/{}>❮</a>
<a href=/preview/{}>.*</a>
<a class=next href=/inscription/{}>❯</a>
</div>.*",
      a.inscription, b.inscription, c.inscription
    ),
  );
}
