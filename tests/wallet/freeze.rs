use {super::*, ord::subcommand::wallet::freeze};

#[test]
fn freezing_rune_fails_if_not_freezable() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: None,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: rune AAAAAAAAAAAAA not freezable\n")
  .run_and_extract_stdout();
}

#[test]
fn freezing_rune_fails_if_freezer_has_not_been_etched() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  let freezer = SpacedRune {
    rune: Rune(RUNE + 1),
    spacers: 0,
  };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: Some(freezer),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: freezer rune AAAAAAAAAAAAB has not been etched\n")
  .run_and_extract_stdout();
}

#[test]
fn freezing_rune_with_no_rune_index_fails() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: `ord wallet freeze` requires index created with `--index-runes` flag\n")
  .run_and_extract_stdout();
}

#[test]
fn freezing_rune_fails_if_no_freezer_balance() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  let freezer = SpacedRune {
    rune: Rune(RUNE + 1),
    spacers: 0,
  };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: Some(freezer),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: freezer,
        supply: "1000".parse().unwrap(),
        premine: "0".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
        freezer: None,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: insufficient `AAAAAAAAAAAAB` balance, 0 in wallet\n")
  .run_and_extract_stdout();
}

#[test]
fn freezing_rune_fails_with_postage_dust() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  let freezer = SpacedRune {
    rune: Rune(RUNE + 1),
    spacers: 0,
  };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: Some(freezer),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: freezer,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: None,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {} --postage 300sat",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: postage below dust limit of 330sat\n")
  .run_and_extract_stdout();
}

#[test]
fn freezing_rune_removes_balance() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  let freezer = SpacedRune {
    rune: Rune(RUNE + 1),
    spacers: 0,
  };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: Some(freezer),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: freezer,
        supply: "500".parse().unwrap(),
        premine: "500".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: None,
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
    *balance.runes.clone().unwrap().first_key_value().unwrap().1,
    Decimal {
      value: 1000,
      scale: 0,
    }
  );

  assert_eq!(
    *balance.runes.clone().unwrap().iter().nth(1).unwrap().1,
    Decimal {
      value: 500,
      scale: 0,
    }
  );

  assert_eq!(balance.runic.unwrap(), 20000);

  let outputs = CommandBuilder::new("--chain regtest --index-runes wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let mut outpoint = OutPoint::null();
  for output in outputs {
    let Some(runes) = output.runes else {
      continue;
    };

    if runes.contains_key(&rune) {
      outpoint = output.output;
      break;
    }
  }

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {} --outpoints {}",
    Rune(RUNE),
    outpoint,
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<freeze::Output>();

  core.mine_blocks(1);

  pretty_assert_eq!(output.rune, rune,);

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(balance.runes.clone().unwrap().get(&rune), None);

  assert_eq!(balance.runic.unwrap(), 19867);
}

#[test]
fn freezing_rune_on_multiple_outpoints_removes_multiple_balances() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  let freezer = SpacedRune {
    rune: Rune(RUNE + 1),
    spacers: 0,
  };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune,
        supply: "1000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: Some(freezer),
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: freezer,
        supply: "500".parse().unwrap(),
        premine: "500".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
        freezer: None,
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
    *balance.runes.clone().unwrap().first_key_value().unwrap().1,
    Decimal {
      value: 1000,
      scale: 0,
    }
  );

  assert_eq!(
    *balance.runes.clone().unwrap().iter().nth(1).unwrap().1,
    Decimal {
      value: 500,
      scale: 0,
    }
  );

  assert_eq!(balance.runic.unwrap(), 20000);

  CommandBuilder::new(format!(
    "--regtest --index-runes wallet send bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 5:{} --fee-rate 1",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::send::Output>();

  core.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  let outpoints: Vec<OutPoint> = balances.runes.get(&rune).unwrap().keys().cloned().collect();
  assert_eq!(outpoints.len(), 2);

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {} --outpoints {} --outpoints {}",
    Rune(RUNE),
    outpoints[0],
    outpoints[1],
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<freeze::Output>();

  core.mine_blocks(1);

  pretty_assert_eq!(output.rune, rune,);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  assert_eq!(balances.runes.get(&rune), None);
}
