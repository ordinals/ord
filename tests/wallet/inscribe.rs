use {super::*, std::ops::Deref};

#[test]
fn inscribe_creates_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let (inscription, _) = inscribe(&rpc_server);

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
    "wallet inscribe --file foo.txt --satpoint {txid}:0:0 --fee-rate 10"
  ))
  .write("foo.txt", [0; 350_000])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>();
}

#[test]
fn metaprotocol_appears_on_inscription_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let inscribe = CommandBuilder::new(format!(
    "wallet inscribe --file foo.txt --metaprotocol foo --satpoint {txid}:0:0 --fee-rate 10"
  ))
  .write("foo.txt", [0; 350_000])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{}", inscribe.inscriptions[0].id),
    r".*<dt>metaprotocol</dt>\s*<dd>foo</dd>.*",
  );
}

#[test]
fn inscribe_fails_if_bitcoin_core_is_too_old() {
  let rpc_server = test_bitcoincore_rpc::builder().version(230000).build();

  CommandBuilder::new("wallet inscribe --file hello.txt --fee-rate 1")
    .write("hello.txt", "HELLOWORLD")
    .expected_exit_code(1)
    .expected_stderr("error: Bitcoin Core 24.0.0 or newer required, current version is 23.0.0\n")
    .rpc_server(&rpc_server)
    .run_and_extract_stdout();
}

#[test]
fn inscribe_no_backup() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);
  assert_eq!(rpc_server.descriptors().len(), 2);

  CommandBuilder::new("wallet inscribe --file hello.txt --no-backup --fee-rate 1")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(rpc_server.descriptors().len(), 2);
}

#[test]
fn inscribe_unknown_file_extension() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file pepe.xyz --fee-rate 1")
    .write("pepe.xyz", [1; 520])
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex(r"error: unsupported file extension `\.xyz`, supported extensions: apng .*\n")
    .run_and_extract_stdout();
}

#[test]
fn inscribe_exceeds_chain_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: content size of 1025 bytes exceeds 1024 byte limit for signet inscriptions\n",
    )
    .run_and_extract_stdout();
}

#[test]
fn regtest_has_no_content_size_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain regtest wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run_and_extract_stdout();
}

#[test]
fn mainnet_has_no_content_size_limit() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Bitcoin)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run_and_extract_stdout();
}

#[test]
fn inscribe_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(
    "wallet inscribe --file degenerate.png --fee-rate 1"
  )
  .rpc_server(&rpc_server)
  .write("degenerate.png", [1; 100])
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run_and_extract_stdout();
}

#[test]
fn refuse_to_reinscribe_sats() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let (_, reveal) = inscribe(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {reveal}:0:0 --file hello.txt --fee-rate 1"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!("error: sat at {reveal}:0:0 already inscribed\n"))
  .run_and_extract_stdout();
}

#[test]
fn refuse_to_inscribe_already_inscribed_utxo() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (inscription, reveal) = inscribe(&rpc_server);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {output}:55555 --file hello.txt --fee-rate 1"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: utxo {output} already inscribed with inscription {inscription} on sat {output}:0\n",
  ))
  .run_and_extract_stdout();
}

#[test]
fn inscribe_with_optional_satpoint_arg() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { inscriptions, .. } = CommandBuilder::new(format!(
    "wallet inscribe --file foo.txt --satpoint {txid}:0:10000 --fee-rate 1"
  ))
  .write("foo.txt", "FOO")
  .rpc_server(&rpc_server)
  .run_and_deserialize_output();
  let inscription = inscriptions[0].id;

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &["--index-sats"]).assert_response_regex(
    "/sat/5000010000",
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

  let output =
    CommandBuilder::new("--index-sats wallet inscribe --file degenerate.png --fee-rate 2.0")
      .write("degenerate.png", [1; 520])
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Inscribe>();

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
  assert_eq!(
    ord::FeeRate::try_from(2.0)
      .unwrap()
      .fee(tx1.vsize() + tx2.vsize())
      .to_sat(),
    output.total_fees
  );
}

#[test]
fn inscribe_with_commit_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--index-sats wallet inscribe --file degenerate.png --commit-fee-rate 2.0 --fee-rate 1",
  )
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>();

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
    .run_and_deserialize_output::<ord::subcommand::wallet::create::Output>();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--wallet foo wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();
}

#[test]
fn inscribe_with_dry_run_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --dry-run --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert!(rpc_server.mempool().is_empty());

  CommandBuilder::new("wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(rpc_server.mempool().len(), 2);
}

#[test]
fn inscribe_with_dry_run_flag_fees_increase() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let total_fee_dry_run =
    CommandBuilder::new("wallet inscribe --dry-run --file degenerate.png --fee-rate 1")
      .write("degenerate.png", [1; 520])
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Inscribe>()
      .total_fees;

  let total_fee_normal =
    CommandBuilder::new("wallet inscribe --dry-run --file degenerate.png --fee-rate 1.1")
      .write("degenerate.png", [1; 520])
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Inscribe>()
      .total_fees;

  assert!(total_fee_dry_run < total_fee_normal);
}

#[test]
fn inscribe_to_specific_destination() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let destination = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .address;

  let txid = CommandBuilder::new(format!(
    "wallet inscribe --destination {} --file degenerate.png --fee-rate 1",
    destination.clone().assume_checked()
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>()
  .reveal;

  let reveal_tx = &rpc_server.mempool()[1]; // item 0 is the commit, item 1 is the reveal.
  assert_eq!(reveal_tx.txid(), txid);
  assert_eq!(
    reveal_tx.output.first().unwrap().script_pubkey,
    destination.payload.script_pubkey()
  );
}

#[test]
fn inscribe_to_address_on_different_network() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "wallet inscribe --destination tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm --file degenerate.png --fee-rate 1"
  )
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .stderr_regex("error: address tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm belongs to network testnet which is different from required bitcoin\n")
  .run_and_extract_stdout();
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

#[test]
fn inscribe_works_with_postage() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file foo.txt --postage 5btc --fee-rate 10".to_string())
    .write("foo.txt", [0; 350])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  let inscriptions = CommandBuilder::new("wallet inscriptions".to_string())
    .write("foo.txt", [0; 350])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::inscriptions::Output>>();

  pretty_assert_eq!(inscriptions[0].postage, 5 * COIN_VALUE);
}

#[test]
fn inscribe_with_non_existent_parent_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let parent_id = "0000000000000000000000000000000000000000000000000000000000000000i0";

  CommandBuilder::new(format!(
    "wallet inscribe --fee-rate 1.0 --parent {parent_id} --file child.png"
  ))
  .write("child.png", [1; 520])
  .rpc_server(&rpc_server)
  .expected_stderr(format!("error: parent {parent_id} does not exist\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscribe_with_parent_inscription_and_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(rpc_server.descriptors().len(), 3);
  let parent_id = parent_output.inscriptions[0].id;

  let commit_tx = &rpc_server.mempool()[0];
  let reveal_tx = &rpc_server.mempool()[1];

  assert_eq!(
    ord::FeeRate::try_from(5.0)
      .unwrap()
      .fee(commit_tx.vsize() + reveal_tx.vsize())
      .to_sat(),
    parent_output.total_fees
  );

  rpc_server.mine_blocks(1);

  let child_output = CommandBuilder::new(format!(
    "wallet inscribe --fee-rate 7.3 --parent {parent_id} --file child.png"
  ))
  .write("child.png", [1; 520])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  assert_eq!(rpc_server.descriptors().len(), 4);
  assert_eq!(parent_id, child_output.parent.unwrap());

  let commit_tx = &rpc_server.mempool()[0];
  let reveal_tx = &rpc_server.mempool()[1];

  assert_eq!(
    ord::FeeRate::try_from(7.3)
      .unwrap()
      .fee(commit_tx.vsize() + reveal_tx.vsize())
      .to_sat(),
    child_output.total_fees
  );

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{}", child_output.parent.unwrap()),
    format!(
      ".*<dt>children</dt>.*<a href=/inscription/{}>.*",
      child_output.inscriptions[0].id
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", child_output.inscriptions[0].id),
    format!(
      ".*<dt>parent</dt>.*<a class=monospace href=/inscription/{}>.*",
      child_output.parent.unwrap()
    ),
  );
}

#[test]
fn reinscribe_with_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let inscribe = CommandBuilder::new("wallet inscribe --file tulip.png --fee-rate 5.0 ")
    .write("tulip.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(rpc_server.descriptors().len(), 3);

  let txid = rpc_server.mine_blocks(1)[0].txdata[2].txid();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  let request = ord_server.request(format!("/content/{}", inscribe.inscriptions[0].id));

  assert_eq!(request.status(), 200);

  let reinscribe = CommandBuilder::new(format!(
    "wallet inscribe --file orchid.png --fee-rate 1.1 --reinscribe --satpoint {txid}:0:0"
  ))
  .write("orchid.png", [1; 520])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &["--index-sats"]);
  let request = ord_server.request(format!("/content/{}", reinscribe.inscriptions[0].id));

  assert_eq!(request.status(), 200);
  ord_server.assert_response_regex(
    format!("/sat/{}", 50 * COIN_VALUE),
    format!(
      ".*<dt>inscriptions</dt>.*<a href=/inscription/{}>.*<a href=/inscription/{}>.*",
      inscribe.inscriptions[0].id, reinscribe.inscriptions[0].id
    ),
  );
}

#[test]
fn with_reinscribe_flag_but_not_actually_a_reinscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  CommandBuilder::new("wallet inscribe --file tulip.png --fee-rate 5.0 ")
    .write("tulip.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  let coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet inscribe --file orchid.png --fee-rate 1.1 --reinscribe --satpoint {coinbase}:0:0"
  ))
  .write("orchid.png", [1; 520])
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .stderr_regex("error: reinscribe flag set but this would not be a reinscription.*")
  .run_and_extract_stdout();
}

#[test]
fn try_reinscribe_without_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let reveal_txid = CommandBuilder::new("wallet inscribe --file tulip.png --fee-rate 5.0 ")
    .write("tulip.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>()
    .reveal;

  assert_eq!(rpc_server.descriptors().len(), 3);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet inscribe --file orchid.png --fee-rate 1.1 --satpoint {reveal_txid}:0:0"
  ))
  .write("orchid.png", [1; 520])
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .stderr_regex(format!(
    "error: sat at {reveal_txid}:0:0 already inscribed.*"
  ))
  .run_and_extract_stdout();
}

#[test]
fn no_metadata_appears_on_inscription_page_if_no_metadata_is_passed() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --fee-rate 1 --file content.png")
      .write("content.png", [1; 520])
      .rpc_server(&rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  assert!(!ord_server
    .request(format!("/inscription/{inscription}"),)
    .text()
    .unwrap()
    .contains("metadata"));
}

#[test]
fn json_metadata_appears_on_inscription_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } = CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --json-metadata metadata.json --file content.png",
  )
  .write("content.png", [1; 520])
  .write("metadata.json", r#"{"foo": "bar", "baz": 1}"#)
  .rpc_server(&rpc_server)
  .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    ".*<dt>metadata</dt>.*<dl><dt>foo</dt><dd>bar</dd><dt>baz</dt><dd>1</dd></dl>.*",
  );
}

#[test]
fn cbor_metadata_appears_on_inscription_page() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } = CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --cbor-metadata metadata.cbor --file content.png",
  )
  .write("content.png", [1; 520])
  .write(
    "metadata.cbor",
    [
      0xA2, 0x63, b'f', b'o', b'o', 0x63, b'b', b'a', b'r', 0x63, b'b', b'a', b'z', 0x01,
    ],
  )
  .rpc_server(&rpc_server)
  .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    ".*<dt>metadata</dt>.*<dl><dt>foo</dt><dd>bar</dd><dt>baz</dt><dd>1</dd></dl>.*",
  );
}

#[test]
fn error_message_when_parsing_json_metadata_is_reasonable() {
  CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --json-metadata metadata.json --file content.png",
  )
  .write("content.png", [1; 520])
  .write("metadata.json", "{")
  .stderr_regex(".*failed to parse JSON metadata.*")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn error_message_when_parsing_cbor_metadata_is_reasonable() {
  CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --cbor-metadata metadata.cbor --file content.png",
  )
  .write("content.png", [1; 520])
  .write("metadata.cbor", [0x61])
  .stderr_regex(".*failed to parse CBOR metadata.*")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_fails_if_batchfile_has_no_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("batch.yaml", "mode: shared-output\ninscriptions: []\n")
    .rpc_server(&rpc_server)
    .stderr_regex(".*batchfile must contain at least one inscription.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_can_create_one_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  metadata: 123\n  metaprotocol: foo",
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let request = ord_server.request(format!("/content/{}", output.inscriptions[0].id));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    r".*<dt>metadata</dt>\s*<dd>\n    123\n  </dd>.*<dt>metaprotocol</dt>\s*<dd>foo</dd>.*",
  );
}

#[test]
fn batch_inscribe_with_multiple_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet inscribe --batch batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0].id));
  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[1].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/png");

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[2].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}

#[test]
fn batch_inscribe_with_multiple_inscriptions_with_parent() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[2].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}

#[test]
fn batch_inscribe_respects_dry_run_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml --dry-run")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n",
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert!(rpc_server.mempool().is_empty());

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0].id));

  assert_eq!(request.status(), 404);
}

#[test]
fn batch_in_same_output_but_different_satpoints() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:10000</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:20000</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_in_same_output_with_non_default_postage() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\npostage: 777\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:777</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:1554</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_in_separate_outputs_with_parent() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_3
    ),
  );
}

#[test]
fn batch_in_separate_outputs_with_parent_and_non_default_postage() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\npostage: 777\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_3
    ),
  );
}

#[test]
fn inscribe_does_not_pick_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks(1)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);

  rpc_server.lock(outpoint);

  CommandBuilder::new("wallet inscribe --file hello.txt --fee-rate 1")
    .rpc_server(&rpc_server)
    .write("hello.txt", "HELLOWORLD")
    .expected_exit_code(1)
    .stderr_regex("error: wallet contains no cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn inscribe_can_compress() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --compress --file foo.txt --fee-rate 1".to_string())
      .write("foo.txt", [0; 350_000])
      .rpc_server(&rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  rpc_server.mine_blocks(1);

  let test_server = TestServer::spawn_with_args(&rpc_server, &[]);

  test_server.sync_server();

  let client = reqwest::blocking::Client::builder()
    .brotli(false)
    .build()
    .unwrap();

  let response = client
    .get(
      test_server
        .url()
        .join(format!("/content/{inscription}",).as_ref())
        .unwrap(),
    )
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
  assert_regex_match!(
    response.text().unwrap(),
    "inscription content type `br` is not acceptable"
  );

  let client = reqwest::blocking::Client::builder()
    .brotli(true)
    .build()
    .unwrap();

  let response = client
    .get(
      test_server
        .url()
        .join(format!("/content/{inscription}",).as_ref())
        .unwrap(),
    )
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.bytes().unwrap().deref(), [0; 350_000]);
}

#[test]
fn inscriptions_are_not_compressed_if_no_space_is_saved_by_compression() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --compress --file foo.txt --fee-rate 1".to_string())
      .write("foo.txt", "foo")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  rpc_server.mine_blocks(1);

  let test_server = TestServer::spawn_with_args(&rpc_server, &[]);

  test_server.sync_server();

  let client = reqwest::blocking::Client::builder()
    .brotli(false)
    .build()
    .unwrap();

  let response = client
    .get(
      test_server
        .url()
        .join(format!("/content/{inscription}",).as_ref())
        .unwrap(),
    )
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.text().unwrap(), "foo");
}

#[test]
fn batch_inscribe_fails_if_invalid_network_destination_address() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  CommandBuilder::new("--regtest wallet inscribe --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("batch.yaml", "mode: separate-outputs\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")
    .rpc_server(&rpc_server)
    .stderr_regex("error: address bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 belongs to network bitcoin which is different from required regtest\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_fails_with_shared_output_and_destination_set() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", "")
    .write("batch.yaml", "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4\n- file: tulip.png")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: individual inscription destinations cannot be set in shared-output mode\n")
    .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_works_with_some_destinations_set_and_others_not() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet inscribe --batch batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: separate-outputs\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4\n- file: tulip.png\n- file: meow.wav\n  destination: bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    ".*
  <dt>address</dt>
  <dd class=monospace>bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4</dd>.*",
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      ".*
  <dt>address</dt>
  <dd class=monospace>{}</dd>.*",
      rpc_server.get_change_addresses()[0]
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    ".*
  <dt>address</dt>
  <dd class=monospace>bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k</dd>.*",
  );
}
