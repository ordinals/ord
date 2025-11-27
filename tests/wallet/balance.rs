use {super::*, ord::decimal::Decimal};

#[test]
fn wallet_balance() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>()
      .cardinal,
    0
  );

  core.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      total: 50 * COIN_VALUE,
    }
  );
}

#[test]
fn inscribed_utxos_are_deducted_from_cardinal() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 0,
      total: 0,
    }
  );

  inscribe(&core, &ord);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 100 * COIN_VALUE - 10_000,
      ordinal: 10_000,
      total: 100 * COIN_VALUE,
    }
  );
}

#[test]
fn unsynced_wallet_fails_with_unindexed_output() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .ord(&ord)
      .core(&core)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      total: 50 * COIN_VALUE,
    }
  );

  let no_sync_ord = TestServer::spawn_with_server_args(&core, &[], &["--no-sync"]);

  inscribe(&core, &ord);

  CommandBuilder::new("wallet balance")
    .ord(&no_sync_ord)
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: `ord server` 4 blocks behind `bitcoind`, consider using `--no-sync` to ignore this error\n")
    .run_and_extract_stdout();

  CommandBuilder::new("wallet --no-sync balance")
    .ord(&no_sync_ord)
    .core(&core)
    .expected_exit_code(1)
    .stderr_regex(r"error: output in wallet but not in ord server: [[:xdigit:]]{64}:\d+.*")
    .run_and_extract_stdout();
}