use {
  super::*,
  bitcoin::Witness,
  ord::{
    runes::{Etching, Mint, Pile},
    subcommand::wallet::{balance, mint},
  },
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

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: rune AAAAAAAAAAAAA has not been etched\n")
  .run_and_extract_stdout();

  bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0, Witness::new())],
    op_return: Some(
      Runestone {
        etching: Some(Etching {
          rune: Some(Rune(RUNE)),
          symbol: Some('*'),
          divisibility: 1,
          mint: Some(Mint {
            limit: Some(1111),
            term: Some(2),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      }
      .encipher(),
    ),
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

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

  assert_eq!(
    output.pile,
    Pile {
      amount: 1111,
      divisibility: 1,
      symbol: Some('*'),
    }
  );

  assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: vec![(
        output.rune.rune,
        vec![(
          OutPoint {
            txid: output.mint,
            vout: 1
          },
          output.pile.amount
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
  .expected_stderr("error: rune AAAAAAAAAAAAA mint ended on block 4\n")
  .run_and_extract_stdout();
}

#[test]
fn minting_rune_fails_if_not_mintable() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  bitcoin_rpc_server.mine_blocks(1);

  bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0, Witness::new())],
    op_return: Some(
      Runestone {
        etching: Some(Etching {
          rune: Some(Rune(RUNE)),
          mint: None,
          ..Default::default()
        }),
        ..Default::default()
      }
      .encipher(),
    ),
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

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

  bitcoin_rpc_server.mine_blocks(1);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let rune = Rune(RUNE);

  let deadline: u32 = 3;

  bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0, Witness::new())],
    op_return: Some(
      Runestone {
        etching: Some(Etching {
          rune: Some(rune),
          mint: Some(Mint {
            deadline: Some(deadline),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      }
      .encipher(),
    ),
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

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
fn minting_rune_does_not_send_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 0);

  bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0, Witness::new())],
    op_return: Some(
      Runestone {
        etching: Some(Etching {
          rune: Some(Rune(RUNE)),
          symbol: Some('*'),
          divisibility: 1,
          mint: Some(Mint {
            limit: Some(1111),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      }
      .encipher(),
    ),
    ..Default::default()
  });

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10000);

  CommandBuilder::new("--chain regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 0);

  assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<balance::Output>(),
    balance::Output {
      cardinal: 0,
      ordinal: 10000,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 10000,
    }
  );

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();

  //  bitcoin_rpc_server.mine_blocks(1);
  //  let balances = CommandBuilder::new("--regtest --index-runes balances")
  //    .bitcoin_rpc_server(&bitcoin_rpc_server)
  //    .ord_rpc_server(&ord_rpc_server)
  //    .run_and_deserialize_output::<ord::subcommand::balances::Output>();
  //
  //  assert_eq!(
  //    CommandBuilder::new("--regtest --index-runes wallet balance")
  //      .bitcoin_rpc_server(&bitcoin_rpc_server)
  //      .ord_rpc_server(&ord_rpc_server)
  //      .run_and_deserialize_output::<balance::Output>(),
  //    balance::Output {
  //      cardinal: 0,
  //      ordinal: 10000,
  //      runic: Some(10000),
  //      runes: Some(vec![(rune, 1000)].into_iter().collect()),
  //      total: 20000,
  //    }
  //  );
}
