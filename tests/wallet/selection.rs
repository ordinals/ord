use super::*;

#[test]
fn inscribe_does_not_select_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  drain(&core, &ord);

  CommandBuilder::new("--regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .core(&core)
    .ord(&ord)
    .expected_exit_code(1)
    .expected_stderr("error: wallet contains no cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_amount_does_not_select_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  drain(&core, &ord);

  CommandBuilder::new("--regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 600sat")
    .core(&core)
    .ord(&ord)
    .expected_exit_code(1)
    .expected_stderr("error: not enough cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_satpoint_does_not_send_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks_with_subsidy(1, 10000);

  let etched = etch(&core, &ord, Rune(RUNE));

  CommandBuilder::new(format!(
    "
        --regtest
        --index-runes
        wallet
        send
        --fee-rate 1
        bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
        {}:0
      ",
    etched.output.rune.unwrap().location.unwrap()
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: runic outpoints may not be sent by satpoint\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_inscription_does_not_select_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let (id, _) = inscribe(&core, &ord);

  drain(&core, &ord);

  CommandBuilder::new(
    format!(
      "
        --regtest
        --index-runes
        wallet
        send
        --postage 10000sat
        --fee-rate 1
        bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
        {id}
      "))
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn mint_does_not_select_inscription() {
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
        premine: "1000".parse().unwrap(),
        supply: "2000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          amount: "1000".parse().unwrap(),
          offset: None,
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

  drain(&core, &ord);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 0 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_does_not_send_inscription() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks_with_subsidy(1, 10000);

  let rune = Rune(RUNE);

  CommandBuilder::new("--chain regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Batch>();

  core.mine_blocks_with_subsidy(1, 10000);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 10000,
      ordinal: 10000,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 20000,
    }
  );

  etch(&core, &ord, rune);

  drain(&core, &ord);

  CommandBuilder::new(format!(
    "
       --chain regtest
       --index-runes
       wallet send
       --postage 11111sat
       --fee-rate 0
       bcrt1pyrmadgg78e38ewfv0an8c6eppk2fttv5vnuvz04yza60qau5va0saknu8k
       1000:{rune}
     ",
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}

#[test]
fn split_does_not_select_inscribed_or_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);

  etch(&core, &ord, rune);

  etch(&core, &ord, Rune(RUNE + 1));

  drain(&core, &ord);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 20000,
      runic: Some(20000),
      runes: Some(
        [
          (SpacedRune { rune, spacers: 0 }, "1000".parse().unwrap()),
          (
            SpacedRune {
              rune: Rune(RUNE + 1),
              spacers: 0
            },
            "1000".parse().unwrap()
          ),
        ]
        .into()
      ),
      total: 40000,
    }
  );

  CommandBuilder::new("--regtest wallet split --fee-rate 0 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write(
      "splits.yaml",
      format!(
        "
outputs:
- address: bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
  value: 20000 sat
  runes:
    {rune}: 1000
"
      ),
    )
    .expected_exit_code(1)
    .expected_stderr("error: not enough cardinal utxos\n")
    .run_and_extract_stdout();
}
