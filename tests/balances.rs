use {super::*, ord::subcommand::balances::Output};

#[test]
fn flag_is_required() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  CommandBuilder::new("--regtest balances")
    .bitcoin_rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: `ord balances` requires index created with `--index-runes` flag\n")
    .run_and_extract_stdout();
}

#[test]
fn no_runes() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let output = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&rpc_server)
    .run_and_deserialize_output::<Output>();

  assert_eq!(
    output,
    Output {
      runes: BTreeMap::new()
    }
  );
}

#[test]
fn with_runes() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let a = etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));
  let b = etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE + 1));

  let output = CommandBuilder::new("--regtest --index-runes balances")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .run_and_deserialize_output::<Output>();

  assert_eq!(
    output,
    Output {
      runes: vec![
        (
          Rune(RUNE),
          vec![(
            OutPoint {
              txid: a.transaction,
              vout: 1
            },
            1000
          )]
          .into_iter()
          .collect()
        ),
        (
          Rune(RUNE + 1),
          vec![(
            OutPoint {
              txid: b.transaction,
              vout: 1
            },
            1000
          )]
          .into_iter()
          .collect()
        ),
      ]
      .into_iter()
      .collect(),
    }
  );
}
