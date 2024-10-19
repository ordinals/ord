use {super::*, ord::decimal::Decimal, ord::subcommand::wallet::mint};

#[test]
fn minting_rune_and_fails_if_after_end() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        symbol: '¢',
        supply: "111.1".parse().unwrap(),
        terms: Some(batch::Terms {
          cap: 1,
          offset: Some(batch::Range {
            end: Some(2),
            start: None,
          }),
          amount: "111.1".parse().unwrap(),
          height: None,
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

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<mint::Output>();

  core.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    output.pile,
    Pile {
      amount: 1111,
      divisibility: 1,
      symbol: Some('¢'),
    }
  );

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        output.rune,
        vec![(
          OutPoint {
            txid: output.mint,
            vout: 1
          },
          output.pile,
        )]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: rune AAAAAAAAAAAAA mint ended on block 10\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_fails_if_not_mintable() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
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

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: rune AAAAAAAAAAAAA not mintable\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_with_no_rune_index_fails() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: `ord wallet mint` requires index created with `--index-runes` flag\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_and_then_sending_works() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "111".parse().unwrap(),
        supply: "132".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: Some(batch::Range {
            end: Some(10),
            start: None,
          }),
          amount: "21".parse().unwrap(),
          height: None,
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

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(
    *balance.runes.unwrap().first_key_value().unwrap().1,
    Decimal {
      value: 111,
      scale: 0,
    }
  );

  assert_eq!(balance.runic.unwrap(), 10000);

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<mint::Output>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(
    *balance.runes.unwrap().first_key_value().unwrap().1,
    Decimal {
      value: 132,
      scale: 0,
    }
  );

  assert_eq!(balance.runic.unwrap(), 20000);

  pretty_assert_eq!(
    output.pile,
    Pile {
      amount: 21,
      divisibility: 0,
      symbol: Some('¢'),
    }
  );

  CommandBuilder::new(format!(
    "--regtest --index-runes wallet send bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 5:{} --fee-rate 1",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::send::Output>();
}

#[test]
fn minting_rune_with_destination() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "21".parse().unwrap(),
        symbol: '¢',
        turbo: false,
        terms: Some(batch::Terms {
          cap: 1,
          offset: Some(batch::Range {
            end: Some(10),
            start: None,
          }),
          amount: "21".parse().unwrap(),
          height: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let destination: Address<NetworkUnchecked> = "bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw"
    .parse()
    .unwrap();

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {} --destination {}",
    Rune(RUNE),
    destination.clone().assume_checked()
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<mint::Output>();

  pretty_assert_eq!(
    output.pile,
    Pile {
      amount: 21,
      divisibility: 0,
      symbol: Some('¢'),
    }
  );

  assert_eq!(
    core.mempool()[0].output[1].script_pubkey,
    destination.assume_checked_ref().script_pubkey()
  );

  core.mine_blocks(1);

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(balance.runic, Some(0));

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes balances")
      .core(&core)
      .run_and_deserialize_output::<ord::subcommand::balances::Output>(),
    ord::subcommand::balances::Output {
      runes: vec![(
        SpacedRune::new(Rune(RUNE), 0),
        vec![(
          OutPoint {
            txid: output.mint,
            vout: 1
          },
          Pile {
            amount: 21,
            divisibility: 0,
            symbol: Some('¢')
          },
        )]
        .into_iter()
        .collect()
      )]
      .into_iter()
      .collect(),
    }
  );
}

#[test]
fn minting_rune_with_postage() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "21".parse().unwrap(),
        symbol: '¢',
        turbo: false,
        terms: Some(batch::Terms {
          cap: 1,
          offset: Some(batch::Range {
            end: Some(10),
            start: None,
          }),
          amount: "21".parse().unwrap(),
          height: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {} --postage 2222sat",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  pretty_assert_eq!(
    output.pile,
    Pile {
      amount: 21,
      divisibility: 0,
      symbol: Some('¢'),
    }
  );

  core.mine_blocks(1);

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(balance.runic.unwrap(), 2222);
}

#[test]
fn minting_rune_with_postage_dust() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "21".parse().unwrap(),
        symbol: '¢',
        turbo: false,
        terms: Some(batch::Terms {
          cap: 1,
          offset: Some(batch::Range {
            end: Some(10),
            start: None,
          }),
          amount: "21".parse().unwrap(),
          height: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {} --postage 300sat",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: postage below dust limit of 330sat\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_is_allowed_when_mint_begins_next_block() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        symbol: '¢',
        supply: "111.1".parse().unwrap(),
        terms: Some(batch::Terms {
          cap: 1,
          offset: Some(batch::Range {
            end: None,
            start: Some(1),
          }),
          amount: "111.1".parse().unwrap(),
          height: None,
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

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<mint::Output>();

  core.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    output.pile,
    Pile {
      amount: 1111,
      divisibility: 1,
      symbol: Some('¢'),
    }
  );

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        output.rune,
        vec![(
          OutPoint {
            txid: output.mint,
            vout: 1
          },
          output.pile,
        )]
        .into_iter()
        .collect()
      ),]
      .into_iter()
      .collect(),
    }
  );
}
