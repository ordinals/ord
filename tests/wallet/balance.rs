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
      runic: None,
      runes: None,
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
      runic: None,
      runes: None,
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
      runic: None,
      runes: None,
      total: 100 * COIN_VALUE,
    }
  );
}

#[test]
fn runic_utxos_are_deducted_from_cardinal() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 0,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 0,
    }
  );

  let rune = Rune(RUNE);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        premine: "1000".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 1 },
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE * 7 - 20_000,
      ordinal: 10000,
      runic: Some(10_000),
      runes: Some(
        vec![(
          SpacedRune { rune, spacers: 1 },
          Decimal {
            value: 1000,
            scale: 0,
          }
        )]
        .into_iter()
        .collect()
      ),
      total: 50 * COIN_VALUE * 7,
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
      runic: None,
      runes: None,
      total: 50 * COIN_VALUE,
    }
  );

  let no_sync_ord = TestServer::spawn_with_server_args(&core, &[], &["--no-sync"]);

  inscribe(&core, &ord);

  CommandBuilder::new("wallet balance")
    .ord(&no_sync_ord)
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: wallet failed to synchronize with `ord server` after 20 attempts\n")
    .run_and_extract_stdout();

  CommandBuilder::new("wallet --no-sync balance")
    .ord(&no_sync_ord)
    .core(&core)
    .expected_exit_code(1)
    .stderr_regex(r"error: output in wallet but not in ord server: [[:xdigit:]]{64}:\d+.*")
    .run_and_extract_stdout();
}

#[test]
fn runic_utxos_are_displayed_with_decimal_amount() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 0,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 0,
    }
  );

  let rune = Rune(RUNE);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 3,
        premine: "1.111".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 1 },
        supply: "2.222".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          amount: "1.111".parse().unwrap(),
          cap: 1,
          ..default()
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE * 7 - 20_000,
      ordinal: 10000,
      runic: Some(10_000),
      runes: Some(
        vec![(
          SpacedRune { rune, spacers: 1 },
          Decimal {
            value: 1111,
            scale: 3,
          }
        )]
        .into_iter()
        .collect()
      ),
      total: 50 * COIN_VALUE * 7,
    }
  );
}
