use {super::*, ord::subcommand::wallet::balance::Output};

#[test]
fn wallet_balance() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>()
      .cardinal,
    0
  );

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      runic: None,
      runes: None,
      total: 50 * COIN_VALUE,
    }
  );
}

#[test]
fn inscribed_utxos_are_deducted_from_cardinal() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 0,
      ordinal: 0,
      runic: None,
      runes: None,
      total: 0,
    }
  );

  inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 100 * COIN_VALUE - 10_000,
      ordinal: 10_000,
      runic: None,
      runes: None,
      total: 100 * COIN_VALUE,
    }
  );
}

#[test]
fn runic_utxos_are_deducted_from_cardinal() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 0,
      ordinal: 0,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 0,
    }
  );

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 100 * COIN_VALUE - 10_000,
      ordinal: 0,
      runic: Some(10_000),
      runes: Some(vec![(Rune(RUNE), 1000)].into_iter().collect()),
      total: 100 * COIN_VALUE,
    }
  );
}
#[test]
fn unsynced_wallet_fails_with_unindexed_output() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .ord_rpc_server(&ord_rpc_server)
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      runic: None,
      runes: None,
      total: 50 * COIN_VALUE,
    }
  );

  let no_sync_ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &["--no-sync"]);

  inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("wallet balance")
    .ord_rpc_server(&no_sync_ord_rpc_server)
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: wallet failed to synchronize with ord server\n")
    .run_and_extract_stdout();

  CommandBuilder::new("wallet --no-sync balance")
    .ord_rpc_server(&no_sync_ord_rpc_server)
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .expected_exit_code(1)
    .stderr_regex(r"error: output in wallet but not in ord server: [[:xdigit:]]{64}:\d+.*")
    .run_and_extract_stdout();
}
