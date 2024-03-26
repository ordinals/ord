use {
  super::*,
  ord::{runes::Pile, subcommand::wallet::mint},
};

#[test]
fn minting_rune_and_fails_if_after_end() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
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
        mint: Some(batch::Mint {
          cap: 1,
          term: Some(2),
          limit: "111.1".parse().unwrap(),
          deadline: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<mint::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
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

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: rune AAAAAAAAAAAAA mint ended on block 11\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_fails_if_not_mintable() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
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
        mint: None,
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: rune AAAAAAAAAAAAA not mintable\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_fails_if_after_deadline() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let rune = Rune(RUNE);
  let deadline = 9;

  batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: SpacedRune { rune, spacers: 0 },
        premine: "0".parse().unwrap(),
        supply: "222.2".parse().unwrap(),
        symbol: '¢',
        mint: Some(batch::Mint {
          cap: 2,
          term: Some(2),
          limit: "111.1".parse().unwrap(),
          deadline: Some(deadline),
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {rune}",
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<mint::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {rune}",
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!("error: rune {rune} mint ended at {deadline}\n"))
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_with_no_rune_index_fails() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest"], &[]);

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: `ord wallet etch` requires index created with `--index-runes` flag\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_and_then_sending_works() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
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
        mint: Some(batch::Mint {
          cap: 1,
          term: Some(10),
          limit: "21".parse().unwrap(),
          deadline: None,
        }),
      }),
      inscriptions: vec![batch::Entry {
        file: "inscription.jpeg".into(),
        ..default()
      }],
      ..default()
    },
  );

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(
    *balance.runes.unwrap().first_key_value().unwrap().1,
    111_u128
  );

  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<mint::Output>();

  bitcoin_rpc_server.mine_blocks(1);

  let balance = CommandBuilder::new("--chain regtest --index-runes wallet balance")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(
    *balance.runes.unwrap().first_key_value().unwrap().1,
    132_u128
  );

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
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output::<ord::subcommand::wallet::send::Output>();
}
