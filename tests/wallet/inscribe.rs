use {
  super::*,
  ord::subcommand::wallet::{create, inscriptions, receive, send},
  std::ops::Deref,
};

#[test]
fn inscribe_creates_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 0);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let request = ord_rpc_server.request(format!("/content/{inscription}"));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "FOO");
}

#[test]
fn inscribe_works_with_huge_expensive_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet inscribe --file foo.txt --satpoint {txid}:0:0 --fee-rate 10"
  ))
  .write("foo.txt", [0; 350_000])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();
}

#[test]
fn metaprotocol_appears_on_inscription_page() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let inscribe = CommandBuilder::new(format!(
    "wallet inscribe --file foo.txt --metaprotocol foo --satpoint {txid}:0:0 --fee-rate 10"
  ))
  .write("foo.txt", [0; 350_000])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", inscribe.inscriptions[0].id),
    r".*<dt>metaprotocol</dt>\s*<dd>foo</dd>.*",
  );
}

#[test]
fn inscribe_fails_if_bitcoin_core_is_too_old() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder().version(230000).build();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  CommandBuilder::new("wallet inscribe --file hello.txt --fee-rate 1")
    .write("hello.txt", "HELLOWORLD")
    .expected_exit_code(1)
    .expected_stderr("error: Bitcoin Core 24.0.0 or newer required, current version is 23.0.0\n")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_extract_stdout();
}

#[test]
fn inscribe_no_backup() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 2);

  CommandBuilder::new("wallet inscribe --file hello.txt --no-backup --fee-rate 1")
    .write("hello.txt", "HELLOWORLD")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 2);
}

#[test]
fn inscribe_unknown_file_extension() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file pepe.xyz --fee-rate 1")
    .write("pepe.xyz", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex(r"error: unsupported file extension `\.xyz`, supported extensions: apng .*\n")
    .run_and_extract_stdout();
}

#[test]
fn inscribe_exceeds_chain_limit() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();

  let ord_rpc_server = TestServer::spawn_with_args(&bitcoin_rpc_server, &["--signet"]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("--chain signet wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: content size of 1025 bytes exceeds 1024 byte limit for signet inscriptions\n",
    )
    .run_and_extract_stdout();
}

#[test]
fn regtest_has_no_content_size_limit() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain regtest wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(".*")
    .run_and_extract_stdout();
}

#[test]
fn mainnet_has_no_content_size_limit() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Bitcoin)
    .build();

  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 1025])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(".*")
    .run_and_extract_stdout();
}

#[test]
fn inscribe_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(
    "wallet inscribe --file degenerate.png --fee-rate 1"
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .write("degenerate.png", [1; 100])
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run_and_extract_stdout();
}

#[test]
fn refuse_to_reinscribe_sats() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (_, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 100);

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {reveal}:0:0 --file hello.txt --fee-rate 1"
  ))
  .write("hello.txt", "HELLOWORLD")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!("error: sat at {reveal}:0:0 already inscribed\n"))
  .run_and_extract_stdout();
}

#[test]
fn refuse_to_inscribe_already_inscribed_utxo() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (inscription, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {output}:55555 --file hello.txt --fee-rate 1"
  ))
  .write("hello.txt", "HELLOWORLD")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: utxo {output} with sat {output}:0 already inscribed with the following inscriptions:\n{inscription}\n",
  ))
  .run_and_extract_stdout();
}

#[test]
fn inscribe_with_optional_satpoint_arg() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { inscriptions, .. } = CommandBuilder::new(format!(
    "wallet inscribe --file foo.txt --satpoint {txid}:0:10000 --fee-rate 1"
  ))
  .write("foo.txt", "FOO")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output();
  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    "/sat/5000010000",
    format!(".*<a href=/inscription/{inscription}>.*"),
  );

  ord_rpc_server.assert_response_regex(format!("/content/{inscription}",), "FOO");
}

#[test]
fn inscribe_with_fee_rate() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output =
    CommandBuilder::new("--index-sats wallet inscribe --file degenerate.png --fee-rate 2.0")
      .write("degenerate.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Inscribe>();

  let tx1 = &bitcoin_rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx1.input {
    fee += bitcoin_rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx1.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx1.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);

  let tx2 = &bitcoin_rpc_server.mempool()[1];
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--index-sats wallet inscribe --file degenerate.png --commit-fee-rate 2.0 --fee-rate 1",
  )
  .write("degenerate.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  let tx1 = &bitcoin_rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx1.input {
    fee += bitcoin_rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx1.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx1.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);

  let tx2 = &bitcoin_rpc_server.mempool()[1];
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  CommandBuilder::new("wallet --name foo create")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<create::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet --name foo inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();
}

#[test]
fn inscribe_with_dry_run_flag() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let inscribe =
    CommandBuilder::new("wallet inscribe --dry-run --file degenerate.png --fee-rate 1")
      .write("degenerate.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Inscribe>();

  assert!(inscribe.commit_psbt.is_some());
  assert!(inscribe.reveal_psbt.is_some());

  assert!(bitcoin_rpc_server.mempool().is_empty());

  let inscribe = CommandBuilder::new("wallet inscribe --file degenerate.png --fee-rate 1")
    .write("degenerate.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert!(inscribe.commit_psbt.is_none());
  assert!(inscribe.reveal_psbt.is_none());

  assert_eq!(bitcoin_rpc_server.mempool().len(), 2);
}

#[test]
fn inscribe_with_dry_run_flag_fees_increase() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let total_fee_dry_run =
    CommandBuilder::new("wallet inscribe --dry-run --file degenerate.png --fee-rate 1")
      .write("degenerate.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Inscribe>()
      .total_fees;

  let total_fee_normal =
    CommandBuilder::new("wallet inscribe --dry-run --file degenerate.png --fee-rate 1.1")
      .write("degenerate.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Inscribe>()
      .total_fees;

  assert!(total_fee_dry_run < total_fee_normal);
}

#[test]
fn inscribe_to_specific_destination() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let destination = CommandBuilder::new("wallet receive")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<receive::Output>()
    .address;

  let txid = CommandBuilder::new(format!(
    "wallet inscribe --destination {} --file degenerate.png --fee-rate 1",
    destination.clone().assume_checked()
  ))
  .write("degenerate.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>()
  .reveal;

  let reveal_tx = &bitcoin_rpc_server.mempool()[1]; // item 0 is the commit, item 1 is the reveal.
  assert_eq!(reveal_tx.txid(), txid);
  assert_eq!(
    reveal_tx.output.first().unwrap().script_pubkey,
    destination.payload.script_pubkey()
  );
}

#[test]
fn inscribe_to_address_on_different_network() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "wallet inscribe --destination tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm --file degenerate.png --fee-rate 1"
  )
  .write("degenerate.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .stderr_regex("error: address tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm belongs to network testnet which is different from required bitcoin\n")
  .run_and_extract_stdout();
}

#[test]
fn inscribe_with_no_limit() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let four_megger = std::iter::repeat(0).take(4_000_000).collect::<Vec<u8>>();
  CommandBuilder::new("wallet inscribe --no-limit degenerate.png --fee-rate 1")
    .write("degenerate.png", four_megger)
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server);
}

#[test]
fn inscribe_works_with_postage() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);
  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file foo.txt --postage 5btc --fee-rate 10".to_string())
    .write("foo.txt", [0; 350])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  let inscriptions = CommandBuilder::new("wallet inscriptions".to_string())
    .write("foo.txt", [0; 350])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  pretty_assert_eq!(inscriptions[0].postage, 5 * COIN_VALUE);
}

#[test]
fn inscribe_with_non_existent_parent_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_id = "0000000000000000000000000000000000000000000000000000000000000000i0";

  CommandBuilder::new(format!(
    "wallet inscribe --fee-rate 1.0 --parent {parent_id} --file child.png"
  ))
  .write("child.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_stderr(format!("error: parent {parent_id} does not exist\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscribe_with_parent_inscription_and_fee_rate() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 --file parent.png")
    .write("parent.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);
  let parent_id = parent_output.inscriptions[0].id;

  let commit_tx = &bitcoin_rpc_server.mempool()[0];
  let reveal_tx = &bitcoin_rpc_server.mempool()[1];

  assert_eq!(
    ord::FeeRate::try_from(5.0)
      .unwrap()
      .fee(commit_tx.vsize() + reveal_tx.vsize())
      .to_sat(),
    parent_output.total_fees
  );

  bitcoin_rpc_server.mine_blocks(1);

  let child_output = CommandBuilder::new(format!(
    "wallet inscribe --fee-rate 7.3 --parent {parent_id} --file child.png"
  ))
  .write("child.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 4);
  assert_eq!(parent_id, child_output.parent.unwrap());

  let commit_tx = &bitcoin_rpc_server.mempool()[0];
  let reveal_tx = &bitcoin_rpc_server.mempool()[1];

  assert_eq!(
    ord::FeeRate::try_from(7.3)
      .unwrap()
      .fee(commit_tx.vsize() + reveal_tx.vsize())
      .to_sat(),
    child_output.total_fees
  );

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", child_output.parent.unwrap()),
    format!(
      ".*<dt>children</dt>.*<a href=/inscription/{}>.*",
      child_output.inscriptions[0].id
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", child_output.inscriptions[0].id),
    format!(
      ".*<dt>parent</dt>.*<a href=/inscription/{}>.*",
      child_output.parent.unwrap()
    ),
  );
}

#[test]
fn reinscribe_with_flag() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 0);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let inscribe = CommandBuilder::new("wallet inscribe --file tulip.png --fee-rate 5.0 ")
    .write("tulip.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[2].txid();

  let request = ord_rpc_server.request(format!("/content/{}", inscribe.inscriptions[0].id));

  assert_eq!(request.status(), 200);

  let reinscribe = CommandBuilder::new(format!(
    "wallet inscribe --file orchid.png --fee-rate 1.1 --reinscribe --satpoint {txid}:0:0"
  ))
  .write("orchid.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  let request = ord_rpc_server.request(format!("/content/{}", reinscribe.inscriptions[0].id));

  assert_eq!(request.status(), 200);
  ord_rpc_server.assert_response_regex(
    format!("/sat/{}", 50 * COIN_VALUE),
    format!(
      ".*<dt>inscriptions</dt>.*<a href=/inscription/{}>.*<a href=/inscription/{}>.*",
      inscribe.inscriptions[0].id, reinscribe.inscriptions[0].id
    ),
  );

  let inscriptions = CommandBuilder::new("wallet inscriptions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscriptions>();

  assert_eq!(inscriptions[0].inscription, inscribe.inscriptions[0].id);
  assert_eq!(inscriptions[1].inscription, reinscribe.inscriptions[0].id);
}

#[test]
fn with_reinscribe_flag_but_not_actually_a_reinscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --file tulip.png --fee-rate 5.0 ")
    .write("tulip.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  let coinbase = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet inscribe --file orchid.png --fee-rate 1.1 --reinscribe --satpoint {coinbase}:0:0"
  ))
  .write("orchid.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .stderr_regex("error: reinscribe flag set but this would not be a reinscription.*")
  .run_and_extract_stdout();
}

#[test]
fn try_reinscribe_without_flag() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let reveal_txid = CommandBuilder::new("wallet inscribe --file tulip.png --fee-rate 5.0 ")
    .write("tulip.png", [1; 520])
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>()
    .reveal;

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet inscribe --file orchid.png --fee-rate 1.1 --satpoint {reveal_txid}:0:0"
  ))
  .write("orchid.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .stderr_regex(format!(
    "error: sat at {reveal_txid}:0:0 already inscribed.*"
  ))
  .run_and_extract_stdout();
}

#[test]
fn no_metadata_appears_on_inscription_page_if_no_metadata_is_passed() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --fee-rate 1 --file content.png")
      .write("content.png", [1; 520])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  assert!(!ord_rpc_server
    .request(format!("/inscription/{inscription}"),)
    .text()
    .unwrap()
    .contains("metadata"));
}

#[test]
fn json_metadata_appears_on_inscription_page() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } = CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --json-metadata metadata.json --file content.png",
  )
  .write("content.png", [1; 520])
  .write("metadata.json", r#"{"foo": "bar", "baz": 1}"#)
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    ".*<dt>metadata</dt>.*<dl><dt>foo</dt><dd>bar</dd><dt>baz</dt><dd>1</dd></dl>.*",
  );
}

#[test]
fn cbor_metadata_appears_on_inscription_page() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

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
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    ".*<dt>metadata</dt>.*<dl><dt>foo</dt><dd>bar</dd><dt>baz</dt><dd>1</dd></dl>.*",
  );
}

#[test]
fn error_message_when_parsing_json_metadata_is_reasonable() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --json-metadata metadata.json --file content.png",
  )
  .write("content.png", [1; 520])
  .write("metadata.json", "{")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .stderr_regex(".*failed to parse JSON metadata.*")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn error_message_when_parsing_cbor_metadata_is_reasonable() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(
    "wallet inscribe --fee-rate 1 --cbor-metadata metadata.cbor --file content.png",
  )
  .write("content.png", [1; 520])
  .write("metadata.cbor", [0x61])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .stderr_regex(".*failed to parse CBOR metadata.*")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_fails_if_batchfile_has_no_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
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

  let output = CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  metadata: 123\n  metaprotocol: foo",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  let output = CommandBuilder::new("wallet inscribe --batch batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
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

  let output = CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml --dry-run")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
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

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\npostage: 777\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
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
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
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

  bitcoin_rpc_server.mine_blocks(1);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>10000</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
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
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\npostage: 777\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
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

  bitcoin_rpc_server.mine_blocks(1);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>777</dd>.*.*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_3
    ),
  );
}

#[test]
fn inscribe_does_not_pick_locked_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let coinbase_tx = &bitcoin_rpc_server.mine_blocks(1)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);

  bitcoin_rpc_server.lock(outpoint);

  CommandBuilder::new("wallet inscribe --file hello.txt --fee-rate 1")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .write("hello.txt", "HELLOWORLD")
    .expected_exit_code(1)
    .stderr_regex("error: wallet contains no cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn inscribe_can_compress() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --compress --file foo.txt --fee-rate 1".to_string())
      .write("foo.txt", [0; 350_000])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.sync_server();

  let client = reqwest::blocking::Client::builder()
    .brotli(false)
    .build()
    .unwrap();

  let response = client
    .get(
      ord_rpc_server
        .url()
        .join(format!("/content/{inscription}",).as_ref())
        .unwrap(),
    )
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
  assert_regex_match!(
    response.text().unwrap(),
    "inscription content encoding `br` is not acceptable. `Accept-Encoding` header not present"
  );

  let client = reqwest::blocking::Client::builder()
    .brotli(true)
    .build()
    .unwrap();

  let response = client
    .get(
      ord_rpc_server
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --compress --file foo.txt --fee-rate 1".to_string())
      .write("foo.txt", "foo")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.sync_server();

  let client = reqwest::blocking::Client::builder()
    .brotli(false)
    .build()
    .unwrap();

  let response = client
    .get(
      ord_rpc_server
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest wallet inscribe --fee-rate 2.1 --batch batch.yaml")
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

  CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", "")
    .write("batch.yaml", "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4\n- file: tulip.png")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: individual inscription destinations cannot be set in `shared-output` or `same-sat` mode\n")
    .run_and_extract_stdout();

  CommandBuilder::new("wallet inscribe --fee-rate 2.1 --batch batch.yaml")
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

  let output = CommandBuilder::new("wallet inscribe --batch batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: separate-outputs\ninscriptions:\n- file: inscription.txt\n  destination: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4\n- file: tulip.png\n- file: meow.wav\n  destination: bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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
      bitcoin_rpc_server.change_addresses()[0]
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

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: same-sat\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("mode: same-sat\nparent: {parent_id}\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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
fn inscribe_with_sat_arg() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(2);

  let Inscribe { inscriptions, .. } = CommandBuilder::new(
    "--index-sats wallet inscribe --file foo.txt --sat 5010000000 --fee-rate 1",
  )
  .write("foo.txt", "FOO")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    "/sat/5010000000",
    format!(".*<a href=/inscription/{inscription}>.*"),
  );

  ord_rpc_server.assert_response_regex(format!("/content/{inscription}",), "FOO");
}

#[test]
fn inscribe_with_sat_arg_fails_if_no_index_or_not_found() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("wallet inscribe --file foo.txt --sat 5010000000 --fee-rate 1")
    .write("foo.txt", "FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: ord index must be built with `--index-sats` to use `--sat`\n")
    .run_and_extract_stdout();

  CommandBuilder::new("--index-sats wallet inscribe --sat 5000000000 --file foo.txt --fee-rate 1")
    .write("foo.txt", "FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&TestServer::spawn_with_server_args(
      &bitcoin_rpc_server,
      &["--index-sats"],
      &[],
    ))
    .expected_exit_code(1)
    .expected_stderr("error: could not find sat `5000000000` in wallet outputs\n")
    .run_and_extract_stdout();
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
      .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(bitcoin_rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscriptions[0].id;

  let output = CommandBuilder::new("--index-sats wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: same-sat\nsat: 5000111111\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n")
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
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

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("mode: same-sat\nsatpoint: {txid}:0:55555\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n", )
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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

  let output = CommandBuilder::new(format!("--index-sats wallet inscribe --fee-rate {set_fee_rate} --batch batch.yaml"))
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: same-sat\nsat: 5000111111\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

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
fn server_can_decompress_brotli() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let Inscribe { inscriptions, .. } =
    CommandBuilder::new("wallet inscribe --compress --file foo.txt --fee-rate 1".to_string())
      .write("foo.txt", [0; 350_000])
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output();

  let inscription = inscriptions[0].id;

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.sync_server();

  let client = reqwest::blocking::Client::builder()
    .brotli(false)
    .build()
    .unwrap();

  let response = client
    .get(
      ord_rpc_server
        .url()
        .join(format!("/content/{inscription}",).as_ref())
        .unwrap(),
    )
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);

  let test_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &["--decompress"]);

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
  assert_eq!(response.bytes().unwrap().deref(), [0; 350_000]);
}

#[test]
fn file_inscribe_with_delegate_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (delegate, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let inscribe = CommandBuilder::new(format!(
    "wallet inscribe --fee-rate 1.0 --delegate {delegate} --file inscription.txt"
  ))
  .write("inscription.txt", "INSCRIPTION")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", inscribe.inscriptions[0].id),
    format!(r#".*<dt>delegate</dt>\s*<dd><a href=/inscription/{delegate}>{delegate}</a></dd>.*"#,),
  );

  ord_rpc_server.assert_response(format!("/content/{}", inscribe.inscriptions[0].id), "FOO");
}

#[test]
fn file_inscribe_with_non_existent_delegate_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let delegate = "0000000000000000000000000000000000000000000000000000000000000000i0";

  CommandBuilder::new(format!(
    "wallet inscribe --fee-rate 1.0 --delegate {delegate} --file child.png"
  ))
  .write("child.png", [1; 520])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_stderr(format!("error: delegate {delegate} does not exist\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn batch_inscribe_with_delegate_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (delegate, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let inscribe = CommandBuilder::new("wallet inscribe --fee-rate 1.0 --batch batch.yaml")
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
    .run_and_deserialize_output::<Inscribe>();

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

  CommandBuilder::new("wallet inscribe --fee-rate 1.0 --batch batch.yaml")
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
      .run_and_deserialize_output::<Inscribe>();

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

  let output = CommandBuilder::new("--index-sats wallet inscribe --fee-rate 1 --batch batch.yaml")
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
    .run_and_deserialize_output::<Inscribe>();

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

  let inscription_1 = output.inscriptions[0];
  let inscription_2 = output.inscriptions[1];
  let inscription_3 = output.inscriptions[2];

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{}", inscription_1.id),
    format!(r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
      50 * COIN_VALUE,
      sat_1,
      inscription_1.location,
    ),
  );

  ord_rpc_server.assert_response_regex(
      format!("/inscription/{}", inscription_2.id),
      format!(r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
         50 * COIN_VALUE,
         sat_2,
         inscription_2.location
      ),
    );

  ord_rpc_server.assert_response_regex(
      format!("/inscription/{}", inscription_3.id),
      format!(r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*<dt>output value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
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

  bitcoin_rpc_server.mine_blocks(3);

  let outpoint_1 = OutPoint {
    txid: CommandBuilder::new(
      "--index-sats wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 25btc",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<send::Output>()
    .txid,
    vout: 0,
  };

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint_2 = OutPoint {
    txid: CommandBuilder::new(
      "--index-sats wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc",
    )
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<send::Output>()
    .txid,
    vout: 0,
  };

  bitcoin_rpc_server.mine_blocks(1);

  let outpoint_3 = OutPoint {
    txid: CommandBuilder::new(
      "--index-sats wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 3btc",
    )
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

  let output = CommandBuilder::new("--index-sats wallet inscribe --fee-rate 1 --batch batch.yaml")
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
    .run_and_deserialize_output::<Inscribe>();

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

  let inscription_1 = output.inscriptions[0];
  let inscription_2 = output.inscriptions[1];
  let inscription_3 = output.inscriptions[2];

  ord_rpc_server.assert_response_regex(
     format!("/inscription/{}", inscription_1.id),
     format!(
       r".*<dt>output value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
       25 * COIN_VALUE,
       sat_1,
       inscription_1.location
     ),
   );

  ord_rpc_server.assert_response_regex(
      format!("/inscription/{}", inscription_2.id),
      format!(
        r".*<dt>output value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
        COIN_VALUE,
        sat_2,
        inscription_2.location
      ),
    );

  ord_rpc_server.assert_response_regex(
         format!("/inscription/{}", inscription_3.id),
         format!(
           r".*<dt>output value</dt>.*<dd>{}</dd>.*<dt>sat</dt>.*<dd>.*{}.*</dd>.*<dt>location</dt>.*<dd class=monospace>{}</dd>.*",
           3 * COIN_VALUE,
           sat_3,
           inscription_3.location
         ),
  );
}
