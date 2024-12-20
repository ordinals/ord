use super::*;

#[test]
fn requires_rune_index() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: `ord wallet split` requires index created with `--index-runes`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn unrecognized_fields_are_forbidden() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write(
      "splits.yaml",
      "
foo:
outputs:
",
    )
    .stderr_regex("error: unknown field `foo`.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();

  CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write(
      "splits.yaml",
      "
outputs:
- address: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
  runes:
  foo:
",
    )
    .stderr_regex(r"error: outputs\[0\]: unknown field `foo`.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn cannot_split_un_etched_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);

  CommandBuilder::new("--regtest wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write(
      "splits.yaml",
      format!(
        "
outputs:
- address: bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
  runes:
    {rune}: 500
"
      ),
    )
    .expected_stderr("error: rune `AAAAAAAAAAAAA` has not been etched\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn simple_split() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  let spaced_rune = SpacedRune { rune, spacers: 1 };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        supply: "100.0".parse().unwrap(),
        divisibility: 1,
        terms: None,
        premine: "100.0".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 1 },
        symbol: '¢',
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
    CommandBuilder::new("--regtest wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 7 * 50 * COIN_VALUE - 20000,
      ordinal: 10000,
      runic: Some(10000),
      runes: Some([(spaced_rune, "100.0".parse().unwrap())].into()),
      total: 7 * 50 * COIN_VALUE,
    }
  );

  let output = CommandBuilder::new(
    "--regtest wallet split --fee-rate 10 --postage 666sat --splits splits.yaml",
  )
  .core(&core)
  .ord(&ord)
  .write(
    "splits.yaml",
    format!(
      "
outputs:
- address: bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
  runes:
    {spaced_rune}: 50.1
"
    ),
  )
  .run_and_deserialize_output::<Split>();

  assert_eq!(output.fee, 2030);

  core.mine_blocks_with_subsidy(1, 0);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 7 * 50 * COIN_VALUE - 10960,
      ordinal: 10000,
      runic: Some(666),
      runes: Some([(spaced_rune, "49.9".parse().unwrap())].into()),
      total: 7 * 50 * COIN_VALUE - 294,
    }
  );

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes balances")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balances>(),
    Balances {
      runes: [(
        spaced_rune,
        [
          (
            OutPoint {
              txid: output.txid,
              vout: 1
            },
            Pile {
              amount: 499,
              divisibility: 1,
              symbol: Some('¢'),
            }
          ),
          (
            OutPoint {
              txid: output.txid,
              vout: 2
            },
            Pile {
              amount: 501,
              divisibility: 1,
              symbol: Some('¢'),
            }
          )
        ]
        .into()
      ),]
      .into(),
    }
  );
}

#[test]
fn oversize_op_returns_are_allowed_with_flag() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);

  let spaced_rune = SpacedRune { rune, spacers: 1 };

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        supply: "10000000000".parse().unwrap(),
        divisibility: 0,
        terms: None,
        premine: "10000000000".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 1 },
        symbol: '¢',
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mut splitfile = String::from("outputs:\n");

  for _ in 0..10 {
    splitfile.push_str(
      "\n- address: bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
  runes:
    AAAAAAAAAAAAA: 1000000000",
    );
  }

  CommandBuilder::new("--regtest wallet split --fee-rate 0 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write("splits.yaml", &splitfile)
    .expected_stderr("error: runestone size 85 over maximum standard OP_RETURN size 83\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();

  let output =
    CommandBuilder::new("--regtest wallet split --fee-rate 0 --splits splits.yaml --no-limit")
      .core(&core)
      .ord(&ord)
      .write("splits.yaml", &splitfile)
      .run_and_deserialize_output::<Split>();

  core.mine_blocks(1);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes balances")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balances>(),
    Balances {
      runes: [(
        spaced_rune,
        (0..10)
          .map(|i| (
            OutPoint {
              txid: output.txid,
              vout: 1 + i,
            },
            Pile {
              amount: 1000000000,
              divisibility: 0,
              symbol: Some('¢'),
            }
          ),)
          .collect()
      )]
      .into(),
    }
  );
}
