use {
  super::*,
  base64::Engine,
  bitcoin::psbt::Psbt,
  ord::subcommand::wallet::{balance, create, send},
  std::collections::BTreeMap,
};

#[test]
fn inscriptions_can_be_sent() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription}",
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .stdout_regex(r".*")
  .run_and_deserialize_output::<send::Output>();

  let txid = bitcoin_rpc_server.mempool()[0].txid();
  assert_eq!(txid, output.txid);

  bitcoin_rpc_server.mine_blocks(1);

  let send_txid = output.txid;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      ".*<h1>Inscription 0</h1>.*<dl>.*
  <dt>content length</dt>
  <dd>3 bytes</dd>
  <dt>content type</dt>
  <dd>text/plain;charset=utf-8</dd>
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {txid}i0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_stderr(format!("error: inscription {txid}i0 not found\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_inscribed_sat() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {inscription}",
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let send_txid = output.txid;

  ord_rpc_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      ".*<h1>Inscription 0</h1>.*<dt>location</dt>.*<dd class=monospace>{send_txid}:0:0</dd>.*",
    ),
  );
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_foo() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("wallet --name foo create")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<create::Output>();

  CommandBuilder::new(format!(
    "wallet --name foo send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {txid}:0:0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder().build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks_with_subsidy(1, 1_000)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz {txid}:0:0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_stderr(
    "error: address tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz belongs to network testnet which is different from required bitcoin\n",
  )
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_ord() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder().build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {txid}:0:0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  assert_eq!(bitcoin_rpc_server.mempool()[0].txid(), output.txid);
}

#[test]
fn send_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10_000)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {txid}:0:0 --file degenerate.png --fee-rate 0"
  ))
  .write("degenerate.png", [1; 100])
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  let txid = bitcoin_rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {txid}:0:0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run_and_extract_stdout();
}

#[test]
fn do_not_send_within_dust_limit_of_an_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (inscription, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {output}:329"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {output}:329 without also sending inscription {inscription} at {output}:0\n"
  ))
  .run_and_extract_stdout();
}

#[test]
fn can_send_after_dust_limit_from_an_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (_, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {output}:330"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();
}

#[test]
fn splitting_merged_inscriptions_is_possible() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(3);

  let inscription = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  // merging 3 inscriptions into one utxo
  let reveal_txid = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, inscription.clone()),
      (2, 0, 0, inscription.clone()),
      (3, 0, 0, inscription.clone()),
    ],
    outputs: 1,
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

  let response = ord_rpc_server.json_request(format!("/output/{}:0", reveal_txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: api::Output = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    api::Output {
      address: None,
      inscriptions: vec![
        InscriptionId {
          txid: reveal_txid,
          index: 0
        },
        InscriptionId {
          txid: reveal_txid,
          index: 1
        },
        InscriptionId {
          txid: reveal_txid,
          index: 2
        },
      ],
      indexed: true,
      runes: Vec::new(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
      script_pubkey: "".to_string(),
      spent: false,
      transaction: reveal_txid.to_string(),
      value: 3 * 50 * COIN_VALUE,
    }
  );

  // try and fail to send first
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i0",
    reveal_txid,
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {reveal_txid}:0:0 without also sending inscription {reveal_txid}i2 at {reveal_txid}:0:{}\n", 100 * COIN_VALUE
  ))
  .run_and_extract_stdout();

  // splitting out last
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i2",
    reveal_txid,
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  // splitting second to last
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i1",
    reveal_txid,
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  // splitting send first
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i0",
    reveal_txid,
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();
}

#[test]
fn inscriptions_cannot_be_sent_by_satpoint() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (_, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {reveal}:0:0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_stderr("error: inscriptions must be sent by inscription ID\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_btc_with_fee_rate() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "wallet send --fee-rate 13.3 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 2btc",
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  let tx = &bitcoin_rpc_server.mempool()[0];

  let mut fee = 0;
  for input in &tx.input {
    fee += bitcoin_rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }

  for output in &tx.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx.vsize() as f64;

  assert!(f64::abs(fee_rate - 13.3) < 0.1);

  assert_eq!(
    Address::from_script(&tx.output[0].script_pubkey, Network::Bitcoin).unwrap(),
    "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .assume_checked()
  );

  assert_eq!(tx.output[0].value, 2 * COIN_VALUE);
}

#[test]
fn send_btc_locks_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (_, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<send::Output>();

  assert!(bitcoin_rpc_server.get_locked().contains(&OutPoint {
    txid: reveal,
    vout: 0,
  }))
}

#[test]
fn send_btc_fails_if_lock_unspent_fails() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .fail_lock_unspent(true)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: failed to lock UTXOs\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn wallet_send_with_fee_rate() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription} --fee-rate 2.0"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  let tx = &bitcoin_rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx.input {
    fee += bitcoin_rpc_server
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
fn user_must_provide_fee_rate_to_send() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription}"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(2)
  .stderr_regex(
    ".*error: the following required arguments were not provided:
.*--fee-rate <FEE_RATE>.*",
  )
  .run_and_extract_stdout();
}

#[test]
fn wallet_send_with_fee_rate_and_target_postage() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription} --fee-rate 2.0 --postage 77000sat"
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  let tx = &bitcoin_rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx.input {
    fee += bitcoin_rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);
  pretty_assert_eq!(tx.output[0].value, 77_000);
}

#[test]
fn send_btc_does_not_send_locked_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let coinbase_tx = &bitcoin_rpc_server.mine_blocks(1)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);

  bitcoin_rpc_server.lock(outpoint);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error:.*")
    .run_and_extract_stdout();
}

#[test]
fn sending_rune_that_has_not_been_etched_is_an_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let coinbase_tx = &bitcoin_rpc_server.mine_blocks(1)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);

  bitcoin_rpc_server.lock(outpoint);

  CommandBuilder::new("--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: rune `FOO` has not been etched\n")
    .run_and_extract_stdout();
}

#[test]
fn sending_rune_with_excessive_precision_is_an_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1.1{}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: excessive precision\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_with_insufficient_balance_is_an_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1001{}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: insufficient `AAAAAAAAAAAAA` balance, only 1000\u{00A0}¢ in wallet\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_works() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000{}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![(
          OutPoint {
            txid: output.txid,
            vout: 2
          },
          1000
        )]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );
}

#[test]
fn sending_spaced_rune_works() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000A•AAAAAAAAAAAA",
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![(
          OutPoint {
            txid: output.txid,
            vout: 2
          },
          1000
        )]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );
}

#[test]
fn sending_rune_with_divisibility_works() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let rune = Rune(RUNE);

  CommandBuilder::new(
    format!(
    "--index-runes --regtest wallet etch --rune {} --divisibility 1 --fee-rate 0 --supply 100 --symbol ¢",
    rune,
    )
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Etch>();

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 10.1{}",
    rune
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![
          (
            OutPoint {
              txid: output.txid,
              vout: 1
            },
            899
          ),
          (
            OutPoint {
              txid: output.txid,
              vout: 2
            },
            101
          )
        ]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );
}

#[test]
fn sending_rune_leaves_unspent_runes_in_wallet() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750{}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![
          (
            OutPoint {
              txid: output.txid,
              vout: 1
            },
            250
          ),
          (
            OutPoint {
              txid: output.txid,
              vout: 2
            },
            750
          )
        ]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );

  let tx = bitcoin_rpc_server.tx(3, 1);

  assert_eq!(tx.txid(), output.txid);

  let address = Address::from_script(&tx.output[1].script_pubkey, Network::Regtest).unwrap();

  assert!(bitcoin_rpc_server
    .change_addresses()
    .iter()
    .any(|change_address| change_address == &address));
}

#[test]
fn sending_rune_creates_transaction_with_expected_runestone() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750{}",
    Rune(RUNE),
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![
          (
            OutPoint {
              txid: output.txid,
              vout: 1
            },
            250
          ),
          (
            OutPoint {
              txid: output.txid,
              vout: 2
            },
            750
          )
        ]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );

  let tx = bitcoin_rpc_server.tx(3, 1);

  assert_eq!(tx.txid(), output.txid);

  assert_eq!(
    Runestone::from_transaction(&tx).unwrap(),
    Runestone {
      default_output: None,
      etching: None,
      edicts: vec![Edict {
        id: RuneId {
          height: 2,
          index: 1
        }
        .into(),
        amount: 750,
        output: 2
      }],
      burn: false,
      claim: None,
    },
  );
}

#[test]
fn error_messages_use_spaced_runes() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  CommandBuilder::new(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1001A•AAAAAAAAAAAA",
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: insufficient `A•AAAAAAAAAAAA` balance, only 1000\u{00A0}¢ in wallet\n")
  .run_and_extract_stdout();

  CommandBuilder::new("--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1F•OO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: rune `FOO` has not been etched\n")
    .run_and_extract_stdout();
}

#[test]
fn sending_rune_does_not_send_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10000);

  let rune = Rune(RUNE);

  CommandBuilder::new("--chain regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10000);

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<balance::Output>(),
    balance::Output {
      cardinal: 10000,
      ordinal: 10000,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 20000,
    }
  );

  CommandBuilder::new(
    format!(
    "--index-runes --regtest wallet etch --rune {} --divisibility 0 --fee-rate 0 --supply 1000 --symbol ¢",
    rune
    )
  )
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<Etch>();

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 0);

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<balance::Output>(),
    balance::Output {
      cardinal: 0,
      ordinal: 10000,
      runic: Some(10000),
      runes: Some(vec![(rune, 1000)].into_iter().collect()),
      total: 20000,
    }
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 0 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000{}",
    rune
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .stderr_regex("error:.*")
  .run_and_extract_stdout();
}

#[test]
fn send_dry_run() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {inscription} --dry-run",
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<send::Output>();

  assert!(bitcoin_rpc_server.mempool().is_empty());
  assert_eq!(
    Psbt::deserialize(
      &base64::engine::general_purpose::STANDARD
        .decode(output.psbt)
        .unwrap()
    )
    .unwrap()
    .fee()
    .unwrap()
    .to_sat(),
    output.fee
  );
  assert_eq!(output.outgoing, Outgoing::InscriptionId(inscription));
}
