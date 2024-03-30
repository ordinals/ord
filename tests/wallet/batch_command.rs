use {super::*, ord::subcommand::wallet::send};

fn receive(
  bitcoin_rpc_server: &test_bitcoincore_rpc::Handle,
  ord_rpc_server: &TestServer,
) -> Address {
  let address = CommandBuilder::new("wallet receive")
    .bitcoin_rpc_server(bitcoin_rpc_server)
    .ord_rpc_server(ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  address
    .require_network(bitcoin_rpc_server.state().network)
    .unwrap()
}

#[test]
fn batch_inscribe_fails_if_batchfile_has_no_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet batch --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("batch.yaml", "mode: shared-output\ninscriptions: []\n")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stderr_regex(".*batchfile must contain at least one inscription.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_can_create_one_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  metadata: 123\n  metaprotocol: foo",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let request = ord_rpc_server.request(format!("/content/{}", output.inscriptions[0].id));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    r".*<dt>metadata</dt>\s*<dd>\n    123\n  </dd>.*<dt>metaprotocol</dt>\s*<dd>foo</dd>.*",
  );
}

#[test]
fn batch_inscribe_with_multiple_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --batch batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let request = ord_rpc_server.request(format!("/content/{}", output.inscriptions[0].id));
  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");

  let request = ord_rpc_server.request(format!("/content/{}", output.inscriptions[1].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/png");

  let request = ord_rpc_server.request(format!("/content/{}", output.inscriptions[2].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}

#[test]
fn batch_inscribe_with_multiple_inscriptions_with_parent() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    r".*<dt>parents</dt>\s*<dd>.*</dd>.*",
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    r".*<dt>parents</dt>\s*<dd>.*</dd>.*",
  );

  let request = ord_rpc_server.request(format!("/content/{}", output.inscriptions[2].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}

#[test]
fn batch_inscribe_respects_dry_run_flag() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --fee-rate 2.1 --batch batch.yaml --dry-run")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert!(bitcoin_rpc_server.mempool().is_empty());

  let request = ord_rpc_server.request(format!("/content/{}", output.inscriptions[0].id));

  assert_eq!(request.status(), 404);
}

#[test]
fn batch_in_same_output_but_different_satpoints() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  let outpoint = output.inscriptions[0].location.outpoint;
  for (i, inscription) in output.inscriptions.iter().enumerate() {
    assert_eq!(
      inscription.location,
      SatPoint {
        outpoint,
        offset: u64::try_from(i).unwrap() * 10_000,
      }
    );
  }

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:10000</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:20000</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_in_same_output_with_non_default_postage() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\npostage: 777\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  let outpoint = output.inscriptions[0].location.outpoint;

  for (i, inscription) in output.inscriptions.iter().enumerate() {
    assert_eq!(
      inscription.location,
      SatPoint {
        outpoint,
        offset: u64::try_from(i).unwrap() * 777,
      }
    );
  }

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:777</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:1554</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_in_separate_outputs_with_parent() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  for inscription in &output.inscriptions {
    assert_eq!(inscription.location.offset, 0);
  }
  let mut outpoints = output
    .inscriptions
    .iter()
    .map(|inscription| inscription.location.outpoint)
    .collect::<Vec<OutPoint>>();
  outpoints.sort();
  outpoints.dedup();
  assert_eq!(outpoints.len(), output.inscriptions.len());

  bitcoin_rpc_server.mine_blocks(1);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_3
    ),
  );
}

#[test]
fn batch_in_separate_outputs_with_parent_and_non_default_postage() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\npostage: 777\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  for inscription in &output.inscriptions {
    assert_eq!(inscription.location.offset, 0);
  }

  let mut outpoints = output
    .inscriptions
    .iter()
    .map(|inscription| inscription.location.outpoint)
    .collect::<Vec<OutPoint>>();
  outpoints.sort();
  outpoints.dedup();
  assert_eq!(outpoints.len(), output.inscriptions.len());

  bitcoin_rpc_server.mine_blocks(1);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_3
    ),
  );
}

#[test]
fn batch_inscribe_fails_if_invalid_network_destination_address() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest wallet batch --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("batch.yaml", "mode: separate-outputs\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stderr_regex("error: address bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 belongs to network bitcoin which is different from required regtest\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_fails_with_shared_output_or_same_sat_and_destination_set() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet batch --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", "")
    .write("batch.yaml", "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4\n- file: tulip.png")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: individual inscription destinations cannot be set in `shared-output` or `same-sat` mode\n")
    .run_and_extract_stdout();

  CommandBuilder::new("wallet batch --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", "")
    .write("batch.yaml", "mode: same-sat\nsat: 5000000000\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4\n- file: tulip.png")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: individual inscription destinations cannot be set in `shared-output` or `same-sat` mode\n")
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_works_with_some_destinations_set_and_others_not() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --batch batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "\
mode: separate-outputs
inscriptions:
- file: inscription.txt
  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
- file: tulip.png
- file: meow.wav
  destination: bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k
",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    ".*
  <dt>address</dt>
  <dd class=monospace>bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4</dd>.*",
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      ".*
  <dt>address</dt>
  <dd class=monospace>{}</dd>.*",
      bitcoin_rpc_server.state().change_addresses[0],
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    ".*
  <dt>address</dt>
  <dd class=monospace>bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k</dd>.*",
  );
}

#[test]
fn batch_same_sat() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: same-sat\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  assert_eq!(
    output.inscriptions[0].location,
    output.inscriptions[1].location
  );
  assert_eq!(
    output.inscriptions[1].location,
    output.inscriptions[2].location
  );

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_same_sat_with_parent() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("mode: same-sat\nparent: {parent_id}\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  assert_eq!(
    output.inscriptions[0].location,
    output.inscriptions[1].location
  );
  assert_eq!(
    output.inscriptions[1].location,
    output.inscriptions[2].location
  );

  bitcoin_rpc_server.mine_blocks(1);

  let txid = output.inscriptions[0].location.outpoint.txid;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", parent_id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0:0</dd>.*",
      txid
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:1:0</dd>.*",
      txid
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:1:0</dd>.*",
      txid
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:1:0</dd>.*",
      txid
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_same_sat_with_satpoint_and_reinscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  let inscription_id = output.inscriptions[0].id;
  let satpoint = output.inscriptions[0].location;

  CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("mode: same-sat\nsatpoint: {}\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n", satpoint)
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex(".*error: sat at .*:0:0 already inscribed.*")
    .run_and_extract_stdout();

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("mode: same-sat\nsatpoint: {}\nreinscribe: true\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n", satpoint)
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  assert_eq!(
    output.inscriptions[0].location,
    output.inscriptions[1].location
  );
  assert_eq!(
    output.inscriptions[1].location,
    output.inscriptions[2].location
  );

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", inscription_id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", inscription_id, output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_inscribe_with_sat_argument_with_parent() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output =
    CommandBuilder::new("--index-sats wallet inscribe --fee-rate 5.0 --file parent.png")
      .write("parent.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("--index-sats wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: same-sat\nsat: 5000111111\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    "/sat/5000111111",
    format!(
      ".*<a href=/inscription/{}>.*<a href=/inscription/{}>.*<a href=/inscription/{}>.*",
      output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id
    ),
  );
}

#[test]
fn batch_inscribe_with_sat_arg_fails_if_wrong_mode() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\nsat: 5000111111\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: neither `sat` nor `satpoint` can be set in `same-sat` mode\n")
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_with_satpoint() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let output = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("mode: same-sat\nsatpoint: {txid}:0:55555\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n", )
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    "/sat/5000055555",
    format!(
      ".*<a href=/inscription/{}>.*<a href=/inscription/{}>.*<a href=/inscription/{}>.*",
      output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id
    ),
  );
}

#[test]
fn batch_inscribe_with_fee_rate() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(2);

  let set_fee_rate = 1.0;

  let output = CommandBuilder::new(format!("--index-sats wallet batch --fee-rate {set_fee_rate} --batch batch.yaml"))
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: same-sat\nsat: 5000111111\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  let commit_tx = &bitcoin_rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &commit_tx.input {
    fee += bitcoin_rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &commit_tx.output {
    fee -= output.value;
  }
  let fee_rate = fee as f64 / commit_tx.vsize() as f64;
  pretty_assert_eq!(fee_rate, set_fee_rate);

  let reveal_tx = &bitcoin_rpc_server.mempool()[1];
  let mut fee = 0;
  for input in &reveal_tx.input {
    fee += &commit_tx.output[input.previous_output.vout as usize].value;
  }
  for output in &reveal_tx.output {
    fee -= output.value;
  }
  let fee_rate = fee as f64 / reveal_tx.vsize() as f64;
  pretty_assert_eq!(fee_rate, set_fee_rate);

  assert_eq!(
    ord::FeeRate::try_from(set_fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + reveal_tx.vsize())
      .to_sat(),
    output.total_fees
  );
}

#[test]
fn batch_inscribe_with_delegate_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (delegate, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let inscribe = CommandBuilder::new("wallet batch --fee-rate 1.0 --batch batch.yaml")
    .write("inscription.txt", "INSCRIPTION")
    .write(
      "batch.yaml",
      format!(
        "mode: shared-output
inscriptions:
- delegate: {delegate}
  file: inscription.txt
"
      ),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", inscribe.inscriptions[0].id),
    format!(r#".*<dt>delegate</dt>\s*<dd><a href=/inscription/{delegate}>{delegate}</a></dd>.*"#,),
  );

  ord_rpc_server.assert_response(format!("/content/{}", inscribe.inscriptions[0].id), "FOO");
}

#[test]
fn batch_inscribe_with_non_existent_delegate_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let delegate = "0000000000000000000000000000000000000000000000000000000000000000i0";

  CommandBuilder::new("wallet batch --fee-rate 1.0 --batch batch.yaml")
    .write("hello.txt", "Hello, world!")
    .write(
      "batch.yaml",
      format!(
        "mode: shared-output
inscriptions:
- delegate: {delegate}
  file: hello.txt
"
      ),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr(format!("error: delegate {delegate} does not exist\n"))
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_with_satpoints_with_parent() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output =
    CommandBuilder::new("--index-sats wallet inscribe --fee-rate 5.0 --file parent.png")
      .write("parent.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  let txids = bitcoin_rpc_server
    .mine_blocks(3)
    .iter()
    .map(|block| block.txdata[0].txid())
    .collect::<Vec<Txid>>();

  let satpoint_1 = SatPoint {
    outpoint: OutPoint {
      txid: txids[0],
      vout: 0,
    },
    offset: 0,
  };

  let satpoint_2 = SatPoint {
    outpoint: OutPoint {
      txid: txids[1],
      vout: 0,
    },
    offset: 0,
  };

  let satpoint_3 = SatPoint {
    outpoint: OutPoint {
      txid: txids[2],
      vout: 0,
    },
    offset: 0,
  };

  let sat_1 = serde_json::from_str::<api::Output>(
    &ord_rpc_server
      .json_request(format!("/output/{}", satpoint_1.outpoint))
      .text()
      .unwrap(),
  )
  .unwrap()
  .sat_ranges
  .unwrap()[0]
    .0;

  let sat_2 = serde_json::from_str::<api::Output>(
    &ord_rpc_server
      .json_request(format!("/output/{}", satpoint_2.outpoint))
      .text()
      .unwrap(),
  )
  .unwrap()
  .sat_ranges
  .unwrap()[0]
    .0;

  let sat_3 = serde_json::from_str::<api::Output>(
    &ord_rpc_server
      .json_request(format!("/output/{}", satpoint_3.outpoint))
      .text()
      .unwrap(),
  )
  .unwrap()
  .sat_ranges
  .unwrap()[0]
    .0;

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("--index-sats wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!(
        r#"
mode: satpoints
parent: {parent_id}
inscriptions:
- file: inscription.txt
  satpoint: {}
- file: tulip.png
  satpoint: {}
- file: meow.wav
  satpoint: {}
"#,
        satpoint_1, satpoint_2, satpoint_3
      ),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", parent_id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0:0</dd>.*",
      output.reveal
    ),
  );

  for inscription in &output.inscriptions {
    assert_eq!(inscription.location.offset, 0);
  }

  let outpoints = output
    .inscriptions
    .iter()
    .map(|inscription| inscription.location.outpoint)
    .collect::<Vec<OutPoint>>();

  assert_eq!(outpoints.len(), output.inscriptions.len());

  let inscription_1 = &output.inscriptions[0];
  let inscription_2 = &output.inscriptions[1];
  let inscription_3 = &output.inscriptions[2];

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", inscription_1.id),
    format!(r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
      50 * COIN_VALUE,
      sat_1,
      inscription_1.location,
    ),
  );

  ord_rpc_server.assert_response_regex(
      format!("/inscription/{}", inscription_2.id),
      format!(r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
         50 * COIN_VALUE,
         sat_2,
         inscription_2.location
      ),
    );

  ord_rpc_server.assert_response_regex(
      format!("/inscription/{}", inscription_3.id),
      format!(r".*<dt>parents</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
        50 * COIN_VALUE,
        sat_3,
        inscription_3.location
      ),
    );
}

#[test]
fn batch_inscribe_with_satpoints_with_different_sizes() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let address_1 = receive(&bitcoin_rpc_server, &ord_rpc_server);
  let address_2 = receive(&bitcoin_rpc_server, &ord_rpc_server);
  let address_3 = receive(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(3);

  let outpoint_1 = OutPoint {
    txid: CommandBuilder::new(format!(
      "--index-sats wallet send --fee-rate 1 {address_1} 25btc"
    ))
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<send::Output>()
    .txid,
    vout: 0,
  };

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint_2 = OutPoint {
    txid: CommandBuilder::new(format!(
      "--index-sats wallet send --fee-rate 1 {address_2} 1btc"
    ))
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<send::Output>()
    .txid,
    vout: 0,
  };

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint_3 = OutPoint {
    txid: CommandBuilder::new(format!(
      "--index-sats wallet send --fee-rate 1 {address_3} 3btc"
    ))
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<send::Output>()
    .txid,
    vout: 0,
  };

  bitcoin_rpc_server.mine_blocks(1);

  let satpoint_1 = SatPoint {
    outpoint: outpoint_1,
    offset: 0,
  };

  let satpoint_2 = SatPoint {
    outpoint: outpoint_2,
    offset: 0,
  };

  let satpoint_3 = SatPoint {
    outpoint: outpoint_3,
    offset: 0,
  };

  let output_1 = serde_json::from_str::<api::Output>(
    &ord_rpc_server
      .json_request(format!("/output/{}", satpoint_1.outpoint))
      .text()
      .unwrap(),
  )
  .unwrap();
  assert_eq!(output_1.value, 25 * COIN_VALUE);

  let output_2 = serde_json::from_str::<api::Output>(
    &ord_rpc_server
      .json_request(format!("/output/{}", satpoint_2.outpoint))
      .text()
      .unwrap(),
  )
  .unwrap();
  assert_eq!(output_2.value, COIN_VALUE);

  let output_3 = serde_json::from_str::<api::Output>(
    &ord_rpc_server
      .json_request(format!("/output/{}", satpoint_3.outpoint))
      .text()
      .unwrap(),
  )
  .unwrap();
  assert_eq!(output_3.value, 3 * COIN_VALUE);

  let sat_1 = output_1.sat_ranges.unwrap()[0].0;
  let sat_2 = output_2.sat_ranges.unwrap()[0].0;
  let sat_3 = output_3.sat_ranges.unwrap()[0].0;

  let output = CommandBuilder::new("--index-sats wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 5])
    .write("meow.wav", [0; 2])
    .write(
      "batch.yaml",
      format!(
        r#"
mode: satpoints
inscriptions:
- file: inscription.txt
  satpoint: {}
- file: tulip.png
  satpoint: {}
- file: meow.wav
  satpoint: {}
"#,
        satpoint_1, satpoint_2, satpoint_3
      ),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Batch>();

  bitcoin_rpc_server.mine_blocks(1);

  for inscription in &output.inscriptions {
    assert_eq!(inscription.location.offset, 0);
  }

  let outpoints = output
    .inscriptions
    .iter()
    .map(|inscription| inscription.location.outpoint)
    .collect::<Vec<OutPoint>>();

  assert_eq!(outpoints.len(), output.inscriptions.len());

  let inscription_1 = &output.inscriptions[0];
  let inscription_2 = &output.inscriptions[1];
  let inscription_3 = &output.inscriptions[2];

  ord_rpc_server.assert_response_regex(
     format!("/inscription/{}", inscription_1.id),
     format!(
       r".*<dt>value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
       25 * COIN_VALUE,
       sat_1,
       inscription_1.location
     ),
   );

  ord_rpc_server.assert_response_regex(
      format!("/inscription/{}", inscription_2.id),
      format!(
        r".*<dt>value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
        COIN_VALUE,
        sat_2,
        inscription_2.location
      ),
    );

  ord_rpc_server.assert_response_regex(
         format!("/inscription/{}", inscription_3.id),
         format!(
           r".*<dt>value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
           3 * COIN_VALUE,
           sat_3,
           inscription_3.location
         ),
  );
}

#[test]
fn batch_inscribe_can_etch_rune() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let batch = batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  let parent = batch.output.inscriptions[0].id;

  let request = ord_rpc_server.request(format!("/content/{parent}"));

  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/jpeg");
  assert_eq!(request.text().unwrap(), "inscription");

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{parent}"),
    r".*<dt>rune</dt>\s*<dd><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></dd>.*",
  );

  ord_rpc_server.assert_response_regex(
    "/rune/AAAAAAAAAAAAA",
    format!(
      r".*<dt>parent</dt>\s*<dd><a class=monospace href=/inscription/{parent}>{parent}</a></dd>.*"
    ),
  );

  assert!(bitcoin_rpc_server.state().is_wallet_address(
    &batch
      .output
      .rune
      .unwrap()
      .destination
      .unwrap()
      .require_network(Network::Regtest)
      .unwrap()
  ));

  assert_eq!(
    bitcoin_rpc_server.tx_by_id(batch.output.reveal).input[0].sequence,
    Sequence::from_height(Runestone::COMMIT_INTERVAL)
  );
}

#[test]
fn batch_inscribe_can_etch_rune_with_offset() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let batch = batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        supply: "10000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 9,
          amount: "1000".parse().unwrap(),
          offset: Some(batch::Range {
            start: Some(10),
            end: Some(20),
          }),
          height: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  let parent = batch.output.inscriptions[0].id;

  let request = ord_rpc_server.request(format!("/content/{parent}"));

  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/jpeg");
  assert_eq!(request.text().unwrap(), "inscription");

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{parent}"),
    r".*<dt>rune</dt>\s*<dd><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></dd>.*",
  );

  ord_rpc_server.assert_response_regex(
    "/rune/AAAAAAAAAAAAA",
    format!(
      r".*<dt>parent</dt>\s*<dd><a class=monospace href=/inscription/{parent}>{parent}</a></dd>.*"
    ),
  );

  assert!(bitcoin_rpc_server.state().is_wallet_address(
    &batch
      .output
      .rune
      .unwrap()
      .destination
      .unwrap()
      .require_network(Network::Regtest)
      .unwrap()
  ));
}

#[test]
fn batch_inscribe_can_etch_rune_with_height() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let batch = batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        supply: "10000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 9,
          amount: "1000".parse().unwrap(),
          height: Some(batch::Range {
            start: Some(10),
            end: Some(20),
          }),
          offset: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  let parent = batch.output.inscriptions[0].id;

  let request = ord_rpc_server.request(format!("/content/{parent}"));

  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/jpeg");
  assert_eq!(request.text().unwrap(), "inscription");

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{parent}"),
    r".*<dt>rune</dt>\s*<dd><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></dd>.*",
  );

  ord_rpc_server.assert_response_regex(
    "/rune/AAAAAAAAAAAAA",
    format!(
      r".*<dt>parent</dt>\s*<dd><a class=monospace href=/inscription/{parent}>{parent}</a></dd>.*"
    ),
  );

  assert!(bitcoin_rpc_server.state().is_wallet_address(
    &batch
      .output
      .rune
      .unwrap()
      .destination
      .unwrap()
      .require_network(Network::Regtest)
      .unwrap()
  ));
}

#[test]
fn etch_existing_rune_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 1,
          },
          supply: "1000".parse().unwrap(),
          premine: "1000".parse().unwrap(),
          symbol: '¢',
          terms: None,
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: rune `AAAAAAAAAAAAA` has already been etched\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn etch_reserved_rune_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune::reserved(0, 0),
            spacers: 0,
          },
          premine: "1000".parse().unwrap(),
          supply: "1000".parse().unwrap(),
          symbol: '¢',
          terms: None,
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: rune `AAAAAAAAAAAAAAAAAAAAAAAAAAA` is reserved\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn etch_sub_minimum_rune_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(0),
            spacers: 0,
          },
          supply: "1000".parse().unwrap(),
          premine: "1000".parse().unwrap(),
          symbol: '¢',
          terms: None,
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: rune is less than minimum for next block: A < ZZQYZPATYGGX\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn etch_requires_rune_index() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "1000".parse().unwrap(),
          premine: "1000".parse().unwrap(),
          symbol: '¢',
          terms: None,
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: etching runes requires index created with `--index-runes`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn etch_divisibility_over_maximum_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 39,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "1000".parse().unwrap(),
          premine: "1000".parse().unwrap(),
          symbol: '¢',
          terms: None,
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: <DIVISIBILITY> must be less than or equal 38\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn etch_mintable_overflow_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: default(),
          premine: default(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 2,
            offset: Some(batch::Range {
              end: Some(2),
              start: None,
            }),
            amount: "340282366920938463463374607431768211455".parse().unwrap(),
            height: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `terms.count` * `terms.amount` over maximum\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn etch_mintable_plus_premine_overflow_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: default(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            offset: Some(batch::Range {
              end: Some(2),
              start: None,
            }),
            amount: "340282366920938463463374607431768211455".parse().unwrap(),
            height: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `premine` + `terms.count` * `terms.amount` over maximum\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn incorrect_supply_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "1".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            offset: Some(batch::Range {
              end: Some(2),
              start: None,
            }),
            amount: "1".parse().unwrap(),
            height: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `supply` not equal to `premine` + `terms.count` * `terms.amount`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn zero_offset_interval_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "2".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            offset: Some(batch::Range {
              end: Some(2),
              start: Some(2),
            }),
            amount: "1".parse().unwrap(),
            height: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `terms.offset.end` must be greater than `terms.offset.start`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn zero_height_interval_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "2".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            height: Some(batch::Range {
              end: Some(2),
              start: Some(2),
            }),
            amount: "1".parse().unwrap(),
            offset: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `terms.height.end` must be greater than `terms.height.start`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn invalid_start_height_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "2".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            height: Some(batch::Range {
              end: None,
              start: Some(0),
            }),
            amount: "1".parse().unwrap(),
            offset: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr(
      "error: `terms.height.start` must be greater than the reveal transaction block height of 8\n",
    )
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn invalid_end_height_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "2".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            height: Some(batch::Range {
              start: None,
              end: Some(0),
            }),
            amount: "1".parse().unwrap(),
            offset: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr(
      "error: `terms.height.end` must be greater than the reveal transaction block height of 8\n",
    )
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn zero_supply_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "0".parse().unwrap(),
          premine: "0".parse().unwrap(),
          symbol: '¢',
          terms: None,
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `supply` must be greater than zero\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn zero_cap_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "1".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 0,
            height: None,
            amount: "1".parse().unwrap(),
            offset: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `terms.cap` must be greater than zero\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn zero_amount_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(RUNE),
            spacers: 0,
          },
          supply: "1".parse().unwrap(),
          premine: "1".parse().unwrap(),
          symbol: '¢',
          terms: Some(batch::Terms {
            cap: 1,
            height: None,
            amount: "0".parse().unwrap(),
            offset: None,
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: `terms.amount` must be greater than zero\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn oversize_runestone_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .write("inscription.txt", "foo")
    .write(
      "batch.yaml",
      serde_yaml::to_string(&batch::File {
        etching: Some(batch::Etching {
          divisibility: 0,
          rune: SpacedRune {
            rune: Rune(6402364363415443603228541259936211926 - 1),
            spacers: 0b00000111_11111111_11111111_11111111,
          },
          supply: u128::MAX.to_string().parse().unwrap(),
          premine: (u128::MAX - 1).to_string().parse().unwrap(),
          symbol: '\u{10FFFF}',
          terms: Some(batch::Terms {
            cap: 1,
            height: Some(batch::Range {
              start: Some(u64::MAX - 1),
              end: Some(u64::MAX),
            }),
            offset: Some(batch::Range {
              start: Some(u64::MAX - 1),
              end: Some(u64::MAX),
            }),
            amount: "1".parse().unwrap(),
          }),
        }),
        inscriptions: vec![batch::Entry {
          file: "inscription.txt".into(),
          ..default()
        }],
        ..default()
      })
      .unwrap(),
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: runestone greater than maximum OP_RETURN size: 125 > 82\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn oversize_runestones_are_allowed_with_no_limit() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--regtest --index-runes wallet batch --fee-rate 0 --dry-run --no-limit --batch batch.yaml",
  )
  .write("inscription.txt", "foo")
  .write(
    "batch.yaml",
    serde_yaml::to_string(&batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(6402364363415443603228541259936211926 - 1),
          spacers: 0b00000111_11111111_11111111_11111111,
        },
        supply: u128::MAX.to_string().parse().unwrap(),
        premine: (u128::MAX - 1).to_string().parse().unwrap(),
        symbol: '\u{10FFFF}',
        terms: Some(batch::Terms {
          cap: 1,
          height: Some(batch::Range {
            start: Some(u64::MAX - 1),
            end: Some(u64::MAX),
          }),
          offset: Some(batch::Range {
            start: Some(u64::MAX - 1),
            end: Some(u64::MAX),
          }),
          amount: "1".parse().unwrap(),
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.txt".into(),
        ..default()
      }],
      ..default()
    })
    .unwrap(),
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Batch>();
}

#[cfg(unix)]
#[test]
fn batch_inscribe_errors_if_pending_etchings() {
  use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
  };

  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let batchfile = batch::File {
    etching: Some(batch::Etching {
      divisibility: 0,
      rune: SpacedRune {
        rune: Rune(RUNE),
        spacers: 0,
      },
      supply: "1000".parse().unwrap(),
      premine: "1000".parse().unwrap(),
      symbol: '¢',
      ..default()
    }),
    inscriptions: vec![batch::Entry {
      file: "inscription.jpeg".into(),
      ..default()
    }],
    ..default()
  };

  let tempdir = Arc::new(TempDir::new().unwrap());

  {
    let mut spawn =
      CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
        .temp_dir(tempdir.clone())
        .write("batch.yaml", serde_yaml::to_string(&batchfile).unwrap())
        .write("inscription.jpeg", "inscription")
        .bitcoin_rpc_server(&bitcoin_rpc_server)
        .ord_rpc_server(&ord_rpc_server)
        .expected_exit_code(1)
        .spawn();

    let mut buffer = String::new();

    BufReader::new(spawn.child.stderr.as_mut().unwrap())
      .read_line(&mut buffer)
      .unwrap();

    assert_eq!(buffer, "Waiting for rune commitment to mature…\n");

    bitcoin_rpc_server.mine_blocks(1);

    signal::kill(
      Pid::from_raw(spawn.child.id().try_into().unwrap()),
      Signal::SIGINT,
    )
    .unwrap();

    buffer.clear();

    BufReader::new(spawn.child.stderr.as_mut().unwrap())
      .read_line(&mut buffer)
      .unwrap();

    assert_eq!(
      buffer,
      "Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.\n"
    );

    spawn.child.wait().unwrap();
  }

  CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
    .temp_dir(tempdir)
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: rune `AAAAAAAAAAAAA` has a pending etching, resume it with `ord wallet resume`\n",
    )
    .run_and_extract_stdout();
}
