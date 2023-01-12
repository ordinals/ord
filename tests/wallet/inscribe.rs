use super::*;

#[test]
fn inscribe_creates_inscription_transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new("wallet inscribe hello.txt")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  let inscription_id = format!("{reveal_txid}i0");

  assert_eq!(rpc_server.descriptors().len(), 3);

  rpc_server.mine_blocks(1);

  let request =
    TestServer::spawn_with_args(&rpc_server, &[]).request(&format!("/content/{inscription_id}"));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "HELLOWORLD");
}

#[test]
fn inscribe_with_satpoint_arg_inscribes_specific_satpoint() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  assert_eq!(rpc_server.descriptors().len(), 2);

  let inscription_id = create_inscription(&rpc_server, "foo.txt");

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request =
    TestServer::spawn_with_args(&rpc_server, &[]).request(&format!("/content/{inscription_id}"));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "HELLOWORLD");
}

#[test]
fn inscribe_fails_if_bitcoin_core_is_too_old() {
  let rpc_server = test_bitcoincore_rpc::builder().version(230000).build();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe hello.txt")
    .write("hello.txt", "HELLOWORLD")
    .expected_exit_code(1)
    .expected_stderr("error: Bitcoin Core 24.0.0 or newer required, current version is 23.0.0\n")
    .rpc_server(&rpc_server)
    .run();
}

#[test]
fn inscribe_no_backup() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);
  assert_eq!(rpc_server.descriptors().len(), 2);

  CommandBuilder::new("wallet inscribe hello.txt --no-backup")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  assert_eq!(rpc_server.descriptors().len(), 2);
}

#[test]
fn inscribe_unknown_file_extension() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe pepe.xyz")
    .write("pepe.xyz", [1; 520])
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex(r"error: unsupported file extension `\.xyz`, supported extensions: apng .*\n")
    .run();
}

#[test]
fn inscribe_exceeds_push_byte_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: content size of 1025 bytes exceeds 1024 byte limit for signet inscriptions\n",
    )
    .run();
}

#[test]
fn regtest_has_no_content_size_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain regtest wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();
}

#[test]
fn mainnet_has_no_content_size_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Bitcoin)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();
}

#[test]
fn inscribe_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks_with_subsidy(1, 800);
  CommandBuilder::new("wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 100])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(
    "wallet inscribe degenerate.png"
  )
  .rpc_server(&rpc_server)
  .write("degenerate.png", [1; 100])
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run();
}

#[test]
fn refuse_to_reinscribe_sats() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 800);
  let stdout = CommandBuilder::new("wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 100])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let first_inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {first_inscription_id}:0:0 hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: sat at {first_inscription_id}:0:0 already inscribed\n"
  ))
  .run();
}

#[test]
fn refuse_to_inscribe_already_inscribed_utxo() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new("wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 100])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  rpc_server.mine_blocks(1);

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  let inscription_utxo = OutPoint {
    txid: reveal_txid_from_inscribe_stdout(&stdout),
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {inscription_utxo}:55555 hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: utxo {inscription_utxo} already inscribed with inscription {reveal_txid}i0 on sat {inscription_utxo}:0\n",
  ))
  .run();
}

#[test]
fn inscribe_with_optional_satpoint_arg() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!("wallet inscribe hello.txt --satpoint {txid}:0:0"))
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  let inscription_id = format!("{reveal_txid}i0");

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &["--index-sats"]).assert_response_regex(
    "/sat/5000000000",
    format!(".*<a href=/inscription/{inscription_id}>.*"),
  );

  TestServer::spawn_with_args(&rpc_server, &[])
    .assert_response_regex(format!("/content/{inscription_id}",), ".*HELLOWORLD.*");
}

#[test]
fn inscribe_with_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--index-sats wallet inscribe degenerate.png --fee-rate 2.0")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let tx = &rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx.input {
    fee += rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);
}

#[test]
fn inscribe_with_wallet_named_foo() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("--wallet foo wallet create")
    .rpc_server(&rpc_server)
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--wallet foo wallet inscribe degenerate.png")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();
}
