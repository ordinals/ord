use {super::*, std::str::FromStr};

mod create;

#[test]
fn sats() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--index-sats wallet sats")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\t{}\t0\tuncommon\n",
      OutPoint::new(second_coinbase, 0),
      50 * COIN_VALUE,
    ))
    .run();
}

#[test]
fn sats_from_tsv_success() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--index-sats wallet sats --tsv foo.tsv")
    .write("foo.tsv", "nvtcsezkbtg")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\tnvtcsezkbtg\n",
      OutPoint::new(second_coinbase, 0),
    ))
    .run();
}

#[test]
fn sats_from_tsv_parse_error() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet sats --tsv foo.tsv")
    .write("foo.tsv", "===")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: failed to parse sat from string \"===\" on line 1: invalid digit found in string\n",
    )
    .run();
}

#[test]
fn sats_from_tsv_file_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  CommandBuilder::new("wallet sats --tsv foo.tsv")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: I/O error reading `.*`\nbecause: .*\n")
    .run();
}

#[test]
fn send_works_on_signet() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain signet --index-sats wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {reveal_txid}"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{reveal_txid}"),
    format!(
      ".*<h1>Inscription {reveal_txid}</h1>.*<dl>.*
  <dt>content size</dt>
  <dd>520 bytes</dd>
  <dt>content type</dt>
  <dd>image/png</dd>
  .*
  <dt>location</dt>
  <dd class=monospace>{send_txid}:0:0</dd>
  .*
</dl>
.*",
    ),
  );
}

#[test]
fn send_unknown_inscription() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain signet wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {txid}"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(format!("error: No inscription found for {txid}\n"))
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_inscribed_sat() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain signet --index-sats wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {reveal_txid}"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex("[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{reveal_txid}"),
    format!(
      ".*<h1>Inscription {reveal_txid}</h1>.*<dt>location</dt>.*<dd class=monospace>{send_txid}:0:0</dd>.*",
    ),
  );
}

#[test]
fn send_on_mainnnet_refuses_to_work_with_wallet_name_foo() {
  let rpc_server = test_bitcoincore_rpc::builder().wallet_name("foo").build();
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(
    format!("wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0")
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: wallet commands may only be used on mainnet with a wallet named `ord` or whose name starts with `ord-`\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(
    "error: Address `tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0` is not valid for mainnet\n",
  )
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_ord() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);
}

#[test]
fn send_on_mainnnet_works_with_wallet_whose_name_starts_with_ord() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .wallet_name("ord-foo")
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);
}

#[test]
fn inscribe_fails_if_bitcoin_core_is_too_old() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .version(230000)
    .build();

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .expected_exit_code(1)
  .expected_stderr("error: Bitcoin Core 24.0.0 or newer required, current version is 23.0.0\n")
  .rpc_server(&rpc_server)
  .run();
}

#[test]
fn inscribe_no_backup() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  create_wallet(&rpc_server);
  assert_eq!(rpc_server.descriptors().len(), 2);

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 hello.txt --no-backup"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  assert_eq!(rpc_server.descriptors().len(), 2);
}

#[test]
fn inscribe() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

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
fn inscribe_unknown_file_extension() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 pepe.xyz"
  ))
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
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain signet wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
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
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 1025])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();
}

#[test]
fn inscribe_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .rpc_server(&rpc_server)
  .write("degenerate.png", [1; 100])
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run();
}

#[test]
fn send_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet send rord1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjac5kr8q {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run();
}

#[test]
fn refuse_to_reinscribe_sats() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let first_inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {first_inscription_id}:0:0 hello.txt"
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
fn do_not_accidentally_send_an_inscription() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let inscription_utxo = OutPoint {
    txid: reveal_txid_from_inscribe_stdout(&stdout),
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--chain regtest wallet send rord1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjac5kr8q {inscription_utxo}:55"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {inscription_utxo}:55 without also sending inscription {inscription_id} at {inscription_utxo}:0\n"
  ))
  .run();
}

#[test]
fn refuse_to_inscribe_already_inscribed_utxo() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  let inscription_utxo = OutPoint {
    txid: reveal_txid_from_inscribe_stdout(&stdout),
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {inscription_utxo}:55555 hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: utxo {inscription_utxo} already inscribed with inscription {inscription_id} on sat {inscription_utxo}:0\n",
  ))
  .run();
}

#[test]
fn inscriptions_cannot_be_sent_by_satpoint() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "--chain regtest wallet send rord1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjac5kr8q {reveal_txid}:0:0"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_stderr("error: inscriptions must be sent by inscription ID\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn receive_cardinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new("wallet receive --cardinal")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();

  assert!(Address::from_str(stdout.trim()).is_ok());
}

#[test]
fn receive_ordinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();

  assert!(stdout.starts_with("ord1"));
}

#[test]
fn outputs() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{outpoint}\t{amount}\n"))
    .run();
}

#[test]
fn outputs_includes_locked_outputs() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  rpc_server.lock(outpoint);

  CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{outpoint}\t{amount}\n"))
    .run();
}

#[test]
fn inscriptions() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .wallet_name("ord-wallet")
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let inscription_id = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new(format!(
      "--chain signet wallet inscribe --satpoint {txid}:0:0 hello.txt"
    ))
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run(),
  );

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{inscription_id}:0:0\n"))
    .run();

  let stdout = CommandBuilder::new("--chain signet wallet receive")
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  let address = stdout.trim();

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send {address} {inscription_id}"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run();

  rpc_server.mine_blocks(1);

  let txid = Txid::from_str(stdout.trim()).unwrap();

  let outpoint = OutPoint::new(txid, 0);

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{outpoint}:0\n"))
    .run();
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let inscription_id = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new("--chain signet wallet inscribe hello.txt")
      .write("hello.txt", "HELLOWORLD")
      .rpc_server(&rpc_server)
      .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
      .run(),
  );

  rpc_server.mine_blocks(1);

  rpc_server.lock(OutPoint {
    txid: inscription_id,
    vout: 0,
  });

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{inscription_id}:0:0\n"))
    .run();
}

#[test]
fn inscribe_with_optional_satpoint_arg() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new("--chain regtest wallet inscribe hello.txt")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &["--index-sats"]).assert_response_regex(
    "/sat/5000000000",
    format!(".*<a href=/inscription/{inscription_id}>.*"),
  );

  TestServer::spawn_with_args(&rpc_server, &[])
    .assert_response_regex(format!("/content/{inscription_id}",), ".*HELLOWORLD.*");
}

#[test]
fn transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n")
    .run();
}

#[test]
fn transactions_with_limit() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n")
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n[[:xdigit:]]{64}\t2\n")
    .run();

  CommandBuilder::new("wallet transactions --limit 1")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n")
    .run();
}

#[test]
fn wallet_balance() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  CommandBuilder::new("--regtest wallet balance")
    .rpc_server(&rpc_server)
    .expected_stdout("0\n")
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest wallet balance")
    .rpc_server(&rpc_server)
    .expected_stdout("5000000000\n")
    .run();
}

#[test]
fn send_btc() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--chain regtest wallet send rord1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjac5kr8q 1btc",
  )
  .rpc_server(&rpc_server)
  .expected_stdout("0000000000000000000000000000000000000000000000000000000000000000\n")
  .run();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bcrt1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjawahtjv"
        .parse::<Address>()
        .unwrap(),
      locked: Vec::new(),
    }]
  )
}

#[test]
fn send_btc_locks_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new("--chain regtest wallet inscribe hello.txt")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--chain regtest wallet send rord1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjac5kr8q 1btc",
  )
  .rpc_server(&rpc_server)
  .expected_stdout("0000000000000000000000000000000000000000000000000000000000000000\n")
  .run();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bcrt1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjawahtjv"
        .parse::<Address>()
        .unwrap(),
      locked: vec![OutPoint {
        txid: inscription_id,
        vout: 0,
      }]
    }]
  )
}

#[test]
fn send_btc_fails_if_lock_unspent_fails() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .fail_lock_unspent(true)
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--chain regtest wallet send rord1qpwxd9k4pm7t5peh8kml7asn2wgmxmfjac5kr8q 1btc",
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: failed to lock ordinal UTXOs\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn refuse_to_send_to_cardinal_address_without_cardinal_flag() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--chain regtest wallet send bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1btc",
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: refusing to send to cardinal adddress, which may be from wallet without sat control; the `--cardinal` flag bypasses this check\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn allow_send_to_cardinal_address_with_cardinal_flag() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "--chain regtest wallet send --cardinal bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1btc",
  )
  .rpc_server(&rpc_server)
  .expected_stdout("0000000000000000000000000000000000000000000000000000000000000000\n")
  .run();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw"
        .parse()
        .unwrap(),
      locked: Vec::new(),
    }]
  )
}
