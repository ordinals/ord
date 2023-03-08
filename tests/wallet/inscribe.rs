use super::*;

#[test]
fn inscribe_creates_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request =
    TestServer::spawn_with_args(&rpc_server, &[]).request(format!("/content/{inscription}"));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "FOO");
}

#[test]
fn inscribe_works_with_huge_expensive_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet inscribe foo.txt --satpoint {txid}:0:0 --fee-rate 10"
  ))
  .write("foo.txt", [0; 350_000])
  .rpc_server(&rpc_server)
  .output::<Inscribe>();
}

#[test]
fn inscribe_fails_if_bitcoin_core_is_too_old() {
  let rpc_server = test_bitcoincore_rpc::builder().version(230000).build();

  CommandBuilder::new("wallet inscribe hello.txt --fee-rate 1")
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

  CommandBuilder::new("wallet inscribe hello.txt --no-backup --fee-rate 1")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .output::<Inscribe>();

  assert_eq!(rpc_server.descriptors().len(), 2);
}

#[test]
fn inscribe_unknown_file_extension() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe pepe.xyz --fee-rate 1")
    .write("pepe.xyz", [1; 520])
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex(r"error: unsupported file extension `\.xyz`, supported extensions: apng .*\n")
    .run();
}

#[test]
fn inscribe_exceeds_chain_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet inscribe degenerate.png --fee-rate 1")
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

  CommandBuilder::new("--chain regtest wallet inscribe degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();
}

#[test]
fn mainnet_has_no_content_size_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Bitcoin)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();
}

#[test]
fn inscribe_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(
    "wallet inscribe degenerate.png --fee-rate 1"
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

  rpc_server.mine_blocks(1);

  let Inscribe { reveal, .. } = inscribe(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {reveal}:0:0 hello.txt --fee-rate 1"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!("error: sat at {reveal}:0:0 already inscribed\n"))
  .run();
}

#[test]
fn refuse_to_inscribe_already_inscribed_utxo() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe {
    reveal,
    inscription,
    ..
  } = inscribe(&rpc_server);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {output}:55555 hello.txt --fee-rate 1"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: utxo {output} already inscribed with inscription {inscription} on sat {output}:0\n",
  ))
  .run();
}

#[test]
fn inscribe_with_optional_satpoint_arg() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { inscription, .. } = CommandBuilder::new(format!(
    "wallet inscribe foo.txt --satpoint {txid}:0:0 --fee-rate 1"
  ))
  .write("foo.txt", "FOO")
  .rpc_server(&rpc_server)
  .output();

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &["--index-sats"]).assert_response_regex(
    "/sat/5000000000",
    format!(".*<a href=/inscription/{inscription}>.*"),
  );

  TestServer::spawn_with_args(&rpc_server, &[])
    .assert_response_regex(format!("/content/{inscription}",), "FOO");
}

#[test]
fn inscribe_with_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--index-sats wallet inscribe degenerate.png --fee-rate 2.0")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .output::<Inscribe>();

  let tx1 = &rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx1.input {
    fee += rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx1.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx1.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);

  let tx2 = &rpc_server.mempool()[1];
  let mut fee = 0;
  for input in &tx2.input {
    fee += &tx1.output[input.previous_output.vout as usize].value;
  }
  for output in &tx2.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx2.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);
}

#[test]
fn inscribe_with_commit_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--index-sats wallet inscribe degenerate.png --commit-fee-rate 2.0 --fee-rate 1",
  )
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .output::<Inscribe>();

  let tx1 = &rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx1.input {
    fee += rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx1.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx1.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);

  let tx2 = &rpc_server.mempool()[1];
  let mut fee = 0;
  for input in &tx2.input {
    fee += &tx1.output[input.previous_output.vout as usize].value;
  }
  for output in &tx2.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx2.vsize() as f64;

  pretty_assert_eq!(fee_rate, 1.0);
}

#[test]
fn inscribe_with_wallet_named_foo() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("--wallet foo wallet create")
    .rpc_server(&rpc_server)
    .output::<Create>();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--wallet foo wallet inscribe degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .output::<Inscribe>();
}

#[test]
fn inscribe_with_dry_run_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --dry-run degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .output::<Inscribe>();

  assert!(rpc_server.mempool().is_empty());

  CommandBuilder::new("wallet inscribe degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .output::<Inscribe>();

  assert_eq!(rpc_server.mempool().len(), 2);
}

#[test]
fn inscribe_with_dry_run_flag_fees_inscrease() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let total_fee_dry_run =
    CommandBuilder::new("wallet inscribe --dry-run degenerate.png --fee-rate 1")
      .write("degenerate.png", [1; 520])
      .rpc_server(&rpc_server)
      .output::<Inscribe>()
      .fees;

  let total_fee_normal =
    CommandBuilder::new("wallet inscribe --dry-run degenerate.png --fee-rate 1.1")
      .write("degenerate.png", [1; 520])
      .rpc_server(&rpc_server)
      .output::<Inscribe>()
      .fees;

  assert!(total_fee_dry_run < total_fee_normal);
}

#[test]
fn inscribe_to_specific_destination() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let destination = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .output::<ord::subcommand::wallet::receive::Output>()
    .address;

  let txid = CommandBuilder::new(format!(
    "wallet inscribe --destination {destination} degenerate.png --fee-rate 1"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .output::<Inscribe>()
  .reveal;

  let reveal_tx = &rpc_server.mempool()[1]; // item 0 is the commit, item 1 is the reveal.
  assert_eq!(reveal_tx.txid(), txid);
  assert_eq!(
    reveal_tx.output.first().unwrap().script_pubkey,
    destination.script_pubkey()
  );
}

#[test]
fn inscribe_with_no_limit() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let four_megger = std::iter::repeat(0).take(4_000_000).collect::<Vec<u8>>();
  CommandBuilder::new("wallet inscribe --no-limit degenerate.png --fee-rate 1")
    .write("degenerate.png", four_megger)
    .rpc_server(&rpc_server);
}
