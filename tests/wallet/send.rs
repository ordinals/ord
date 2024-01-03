use {super::*, ord::subcommand::wallet::send::Output, std::collections::BTreeMap};

#[test]
fn inscriptions_can_be_sent() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription}",
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run_and_deserialize_output::<Output>();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(txid, output.transaction);

  rpc_server.mine_blocks(1);

  let send_txid = output.transaction;

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
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
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {txid}i0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(format!("error: inscription {txid}i0 not found\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_inscribed_sat() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv {inscription}",
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let send_txid = output.transaction;

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      ".*<h1>Inscription 0</h1>.*<dt>location</dt>.*<dd class=monospace>{send_txid}:0:0</dd>.*",
    ),
  );
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_foo() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("wallet --name foo create")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::create::Output>();

  CommandBuilder::new(format!(
    "wallet --name foo send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(
    "error: address tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz belongs to network testnet which is different from required bitcoin\n",
  )
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_ord() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  let output = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  assert_eq!(rpc_server.mempool()[0].txid(), output.transaction);
}

#[test]
fn send_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks_with_subsidy(1, 10_000)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {txid}:0:0 --file degenerate.png --fee-rate 0"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run_and_extract_stdout();
}

#[test]
fn do_not_send_within_dust_limit_of_an_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (inscription, reveal) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {output}:329"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {output}:329 without also sending inscription {inscription} at {output}:0\n"
  ))
  .run_and_extract_stdout();
}

#[test]
fn can_send_after_dust_limit_from_an_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (_, reveal) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {output}:330"
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();
}

#[test]
fn splitting_merged_inscriptions_is_possible() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(3);

  let inscription = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  // merging 3 inscriptions into one utxo
  let reveal_txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, inscription.clone()),
      (2, 0, 0, inscription.clone()),
      (3, 0, 0, inscription.clone()),
    ],
    outputs: 1,
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let server =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"]);

  let response = server.json_request(format!("/output/{}:0", reveal_txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: OutputJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    OutputJson {
      value: 3 * 50 * COIN_VALUE,
      script_pubkey: "".to_string(),
      address: None,
      transaction: reveal_txid.to_string(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
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
      runes: BTreeMap::new(),
    }
  );

  // try and fail to send first
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i0",
    reveal_txid,
  ))
  .rpc_server(&rpc_server)
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
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  // splitting second to last
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i1",
    reveal_txid,
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  // splitting send first
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {}i0",
    reveal_txid,
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();
}

#[test]
fn inscriptions_cannot_be_sent_by_satpoint() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let (_, reveal) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {reveal}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr("error: inscriptions must be sent by inscription ID\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_btc_with_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "wallet send --fee-rate 13.3 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc",
  )
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

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

  assert!(f64::abs(fee_rate - 13.3) < 0.1);

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked(),
      locked: Vec::new(),
    }]
  );
}

#[test]
fn send_btc_locks_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let (_, reveal) = inscribe(&rpc_server);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Output>();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked(),
      locked: vec![OutPoint {
        txid: reveal,
        vout: 0,
      }]
    }]
  )
}

#[test]
fn send_btc_fails_if_lock_unspent_fails() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .fail_lock_unspent(true)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .rpc_server(&rpc_server)
    .expected_stderr("error: failed to lock UTXOs\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn wallet_send_with_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription} --fee-rate 2.0"
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

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
fn user_must_provide_fee_rate_to_send() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription}"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(2)
  .stderr_regex(
    ".*error: the following required arguments were not provided:
.*--fee-rate <FEE_RATE>.*",
  )
  .run_and_extract_stdout();
}

#[test]
fn wallet_send_with_fee_rate_and_target_postage() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription} --fee-rate 2.0 --postage 77000sat"
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

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
  pretty_assert_eq!(tx.output[0].value, 77_000);
}

#[test]
fn send_btc_does_not_send_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks(1)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);

  rpc_server.lock(outpoint);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error:.*")
    .run_and_extract_stdout();
}

#[test]
fn sending_rune_that_has_not_been_etched_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks(1)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);

  rpc_server.lock(outpoint);

  CommandBuilder::new("--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1FOO")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: rune `FOO` has not been etched\n")
    .run_and_extract_stdout();
}

#[test]
fn sending_rune_with_excessive_precision_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1.1{}",
    Rune(RUNE)
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: excessive precision\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_with_insufficient_balance_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1001{}",
    Rune(RUNE)
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: insufficient `AAAAAAAAAAAAA` balance, only 1000\u{00A0}¢ in wallet\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_works() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000{}",
    Rune(RUNE)
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![(
          OutPoint {
            txid: output.transaction,
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
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000A•AAAAAAAAAAAA",
  )
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![(
          OutPoint {
            txid: output.transaction,
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
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let rune = Rune(RUNE);

  CommandBuilder::new(
    format!(
    "--index-runes --regtest wallet etch --rune {} --divisibility 1 --fee-rate 0 --supply 100 --symbol ¢",
    rune,
    )
  )
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Etch>();

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 10.1{}",
    rune
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![
          (
            OutPoint {
              txid: output.transaction,
              vout: 1
            },
            899
          ),
          (
            OutPoint {
              txid: output.transaction,
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
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750{}",
    Rune(RUNE)
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        Rune(RUNE),
        vec![
          (
            OutPoint {
              txid: output.transaction,
              vout: 1
            },
            250
          ),
          (
            OutPoint {
              txid: output.transaction,
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

  let tx = rpc_server.tx(3, 1);

  assert_eq!(tx.txid(), output.transaction);

  let address = Address::from_script(&tx.output[1].script_pubkey, Network::Regtest).unwrap();

  assert!(rpc_server
    .change_addresses()
    .iter()
    .any(|change_address| change_address == &address));
}

#[test]
fn sending_rune_creates_transaction_with_expected_runestone() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  let rune = Rune(RUNE);

  etch(&rpc_server, rune);

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750{}",
    rune,
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        rune,
        vec![
          (
            OutPoint {
              txid: output.transaction,
              vout: 1
            },
            250
          ),
          (
            OutPoint {
              txid: output.transaction,
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

  let tx = rpc_server.tx(3, 1);

  assert_eq!(tx.txid(), output.transaction);

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
    },
  );
}

#[test]
fn error_messages_use_spaced_runes() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  CommandBuilder::new(
    "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1001A•AAAAAAAAAAAA",
  )
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: insufficient `A•AAAAAAAAAAAA` balance, only 1000\u{00A0}¢ in wallet\n")
  .run_and_extract_stdout();

  CommandBuilder::new("--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1F•OO")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: rune `FOO` has not been etched\n")
    .run_and_extract_stdout();
}

#[test]
fn sending_rune_does_not_send_inscription() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  let rune = Rune(RUNE);

  CommandBuilder::new("--chain regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>(),
    ord::subcommand::wallet::balance::Output {
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
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Etch>();

  rpc_server.mine_blocks_with_subsidy(1, 0);

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>(),
    ord::subcommand::wallet::balance::Output {
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
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .stderr_regex("error:.*")
  .run_and_extract_stdout();
}
