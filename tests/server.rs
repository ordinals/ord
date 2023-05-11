use {super::*, crate::command_builder::ToArgs};

#[test]
fn run() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let builder = CommandBuilder::new(format!("server --address 127.0.0.1 --http-port {port}"))
    .rpc_server(&rpc_server);

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
      ".*<meta property=og:title content='Inscription 0'>.*
.*<meta property=og:image content='https://.*/favicon.png'>.*
.*<meta property=twitter:card content=summary>.*
<h1>Inscription 0</h1>
.*<iframe .* src=/preview/{inscription}></iframe>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{inscription}</dd>
  <dt>address</dt>
  <dd class=monospace>bc1.*</dd>
  <dt>output value</dt>
  <dd>10000</dd>
  <dt>preview</dt>
  <dd><a href=/preview/{inscription}>link</a></dd>
  <dt>content</dt>
  <dd><a href=/content/{inscription}>link</a></dd>
  <dt>content length</dt>
  <dd>3 bytes</dd>
  <dt>content type</dt>
  <dd>text/plain;charset=utf-8</dd>
  <dt>timestamp</dt>
  <dd><time>1970-01-01 00:00:02 UTC</time></dd>
  <dt>genesis height</dt>
  <dd><a href=/block/2>2</a></dd>
  <dt>genesis fee</dt>
  <dd>138</dd>
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
    "wallet send --fee-rate 1 bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {inscription}"
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

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).request(format!("/content/{inscription}"));

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(
    response.headers().get("content-security-policy").unwrap(),
    "default-src 'unsafe-eval' 'unsafe-inline' data:"
  );
  assert_eq!(response.bytes().unwrap(), "FOO");
}

#[test]
fn home_page_includes_latest_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/",
    format!(
      ".*<h2>Latest Inscriptions</h2>
<div class=thumbnails>
  <a href=/inscription/{inscription}><iframe .*></a>
</div>.*",
    ),
  );
}

#[test]
fn home_page_inscriptions_are_sorted() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let mut inscriptions = String::new();

  for _ in 0..8 {
    let Inscribe { inscription, .. } = inscribe(&rpc_server);
    inscriptions.insert_str(
      0,
      &format!("\n  <a href=/inscription/{inscription}><iframe .*></a>"),
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

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/inscriptions",
    format!(
      ".*<h1>Inscriptions</h1>
<div class=thumbnails>
  <a href=/inscription/{inscription}>.*</a>
</div>
.*",
    ),
  );
}

#[test]
fn inscriptions_page_is_sorted() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let mut inscriptions = String::new();

  for _ in 0..8 {
    let Inscribe { inscription, .. } = inscribe(&rpc_server);
    inscriptions.insert_str(0, &format!(".*<a href=/inscription/{inscription}>.*"));
  }

  TestServer::spawn_with_args(&rpc_server, &[])
    .assert_response_regex("/inscriptions", &inscriptions);
}

#[test]
fn inscriptions_page_has_next_and_previous() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe { inscription: a, .. } = inscribe(&rpc_server);
  let Inscribe { inscription: b, .. } = inscribe(&rpc_server);
  let Inscribe { inscription: c, .. } = inscribe(&rpc_server);

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    format!("/inscription/{b}"),
    format!(
      ".*<h1>Inscription 1</h1>.*
<div class=inscription>
<a class=prev href=/inscription/{a}>❮</a>
<iframe .* src=/preview/{b}></iframe>
<a class=next href=/inscription/{c}>❯</a>
</div>.*",
    ),
  );
}

#[test]
fn expected_sat_time_is_rounded() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    "/sat/2099999997689999",
    r".*<dt>timestamp</dt><dd><time>.* \d+:\d+:\d+ UTC</time> \(expected\)</dd>.*",
  );
}

#[test]
fn server_runs_with_rpc_user_and_pass_as_env_vars() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();
  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let mut child = Command::new(executable_path("ord"))
    .args(format!(
      "--rpc-url {} --bitcoin-data-dir {} --data-dir {} server --http-port {port} --address 127.0.0.1",
      rpc_server.url(),
      tempdir.path().display(),
      tempdir.path().display()).to_args()
      )
      .env("ORD_BITCOIN_RPC_PASS", "bar")
      .env("ORD_BITCOIN_RPC_USER", "foo")
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

  rpc_server.mine_blocks(1);

  let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/block-count")).unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.text().unwrap(), "2");

  child.kill().unwrap();
}

#[test]
fn missing_credentials() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("--bitcoin-rpc-user foo server")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: no bitcoind rpc password specified\n")
    .run();

  CommandBuilder::new("--bitcoin-rpc-pass bar server")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: no bitcoind rpc user specified\n")
    .run();
}
