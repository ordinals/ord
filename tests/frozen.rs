use super::*;

#[test]
fn flag_is_required() {
  let core = mockcore::builder().network(Network::Regtest).build();

  CommandBuilder::new("--regtest frozen")
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: `ord frozen` requires index created with `--index-runes` flag\n")
    .run_and_extract_stdout();
}

#[test]
fn no_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let output = CommandBuilder::new("--regtest --index-runes frozen")
    .core(&core)
    .run_and_deserialize_output::<Frozen>();

  assert_eq!(
    output,
    Frozen {
      frozen_runes: BTreeMap::new()
    }
  );
}

#[test]
fn with_frozen_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

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

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet freeze --fee-rate 1 --rune {} --outpoints {}",
    Rune(RUNE),
    outpoint,
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::freeze::Output>();

  core.mine_blocks(1);

  let output = CommandBuilder::new("--regtest --index-runes frozen")
    .core(&core)
    .run_and_deserialize_output::<Frozen>();

  assert_eq!(
    output,
    Frozen {
      frozen_runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [(
          outpoint,
          Pile {
            amount: 10000,
            divisibility: 1,
            symbol: Some('¢')
          },
        )]
        .into()
      )]
      .into()
    }
  );
}
