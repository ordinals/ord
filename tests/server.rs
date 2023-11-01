use {
  super::*, crate::command_builder::ToArgs, ciborium::value::Integer,
  ord::subcommand::wallet::send::Output,
};

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

  let (inscription, reveal) = inscribe(&rpc_server);

  let ethereum_teleburn_address = CommandBuilder::new(format!("teleburn {inscription}"))
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::teleburn::Output>()
    .ethereum;

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
  <dt>ethereum teleburn address</dt>
  <dd>{ethereum_teleburn_address}</dd>
</dl>.*",
    ),
  );
}

#[test]
fn inscription_appears_on_reveal_transaction_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (_, reveal) = inscribe(&rpc_server);

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

  let (inscription, reveal) = inscribe(&rpc_server);

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

  let (inscription, reveal) = inscribe(&rpc_server);

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
  .run_and_deserialize_output::<Output>()
  .transaction;

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      r".*<h1>Inscription 0</h1>.*<dt>address</dt>\s*<dd class=monospace>bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv</dd>.*<dt>location</dt>\s*<dd class=monospace>{txid}:0:0</dd>.*",
    ),
  )
}

#[test]
fn inscription_content() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).request(format!("/content/{inscription}"));

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(
    response
      .headers()
      .get_all("content-security-policy")
      .into_iter()
      .collect::<Vec<&http::HeaderValue>>(),
    &[
      "default-src 'self' 'unsafe-eval' 'unsafe-inline' data: blob:",
      "default-src *:*/content/ *:*/blockheight *:*/blockhash *:*/blockhash/ *:*/blocktime *:*/r/ 'unsafe-eval' 'unsafe-inline' data: blob:",
    ]
  );
  assert_eq!(response.bytes().unwrap(), "FOO");
}

#[test]
fn inscription_metadata() {
  let metadata = r#"{"foo":"bar","baz":1}"#;
  let mut encoded_metadata = Vec::new();
  let cbor_map = ciborium::value::Value::Map(vec![
    (
      ciborium::value::Value::Text("foo".into()),
      ciborium::value::Value::Text("bar".into()),
    ),
    (
      ciborium::value::Value::Text("baz".into()),
      ciborium::value::Value::Integer(Integer::from(1)),
    ),
  ]);
  ciborium::ser::into_writer(&cbor_map, &mut encoded_metadata).unwrap();

  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let inscription_id = CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --json-metadata metadata.json --file foo.txt",
  )
  .write("foo.txt", "FOO")
  .write("metadata.json", metadata)
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>()
  .inscriptions
  .get(0)
  .unwrap()
  .id;

  rpc_server.mine_blocks(1);

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).request(format!("/r/metadata/{inscription_id}"));

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/json"
  );
  assert_eq!(
    response.text().unwrap(),
    format!("\"{}\"", hex::encode(encoded_metadata))
  );
}

#[test]
fn inscriptions_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (inscription, _) = inscribe(&rpc_server);

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

  let mut regex = String::new();

  for _ in 0..8 {
    let (inscription, _) = inscribe(&rpc_server);
    regex.insert_str(0, &format!(".*<a href=/inscription/{inscription}>.*"));
  }

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex("/inscriptions", &regex);
}

#[test]
fn inscriptions_page_has_next_and_previous() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (a, _) = inscribe(&rpc_server);
  let (b, _) = inscribe(&rpc_server);
  let (c, _) = inscribe(&rpc_server);

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

  for i in 0.. {
    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/blockcount")).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    if response.text().unwrap() == "2" {
      break;
    }

    if i == 400 {
      panic!("server failed to sync");
    }

    thread::sleep(Duration::from_millis(25));
  }

  child.kill().unwrap();
}

#[test]
fn missing_credentials() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("--bitcoin-rpc-user foo server")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: no bitcoind rpc password specified\n")
    .run_and_extract_stdout();

  CommandBuilder::new("--bitcoin-rpc-pass bar server")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: no bitcoind rpc user specified\n")
    .run_and_extract_stdout();
}

#[test]
fn all_endpoints_in_recursive_directory_return_json() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(2);

  let server = TestServer::spawn_with_args(&rpc_server, &[]);

  assert_eq!(server.request("/r/blockheight").json::<u64>().unwrap(), 2);

  assert_eq!(server.request("/r/blocktime").json::<u64>().unwrap(), 2);

  assert_eq!(
    server.request("/r/blockhash").json::<String>().unwrap(),
    "70a93647a8d559c7e7ff2df9bd875f5b726a2ff8ca3562003d257df5a4c47ae2"
  );

  assert_eq!(
    server.request("/r/blockhash/0").json::<String>().unwrap(),
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
  );

  assert!(server.request("/blockhash").json::<String>().is_err());

  assert!(server.request("/blockhash/2").json::<String>().is_err());
}
