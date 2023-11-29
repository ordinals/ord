use {super::*, ord::subcommand::runes::Output};

#[test]
fn flag_is_required() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  CommandBuilder::new("--regtest runes")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: `ord runes` requires index created with `--index-runes-pre-alpha-i-agree-to-get-rekt` flag\n")
    .run_and_extract_stdout();
}

#[test]
fn no_runes() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  assert_eq!(
    CommandBuilder::new("--index-runes-pre-alpha-i-agree-to-get-rekt --regtest runes")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      runes: BTreeMap::new(),
    }
  );
}

#[test]
fn one_rune() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  let etch = etch(&rpc_server, Rune(RUNE));

  assert_eq!(
    CommandBuilder::new("--index-runes-pre-alpha-i-agree-to-get-rekt --regtest runes")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      runes: vec![(
        Rune(RUNE),
        RuneInfo {
          burned: 0,
          divisibility: 0,
          end: None,
          etching: etch.transaction,
          height: 2,
          id: RuneId {
            height: 2,
            index: 1
          },
          index: 1,
          limit: None,
          number: 0,
          rune: Rune(RUNE),
          supply: 1000,
          symbol: Some('¢'),
          timestamp: ord::timestamp(2),
        }
      )]
      .into_iter()
      .collect(),
    }
  );
}

#[test]
fn two_runes() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  let a = etch(&rpc_server, Rune(RUNE));
  let b = etch(&rpc_server, Rune(RUNE + 1));

  assert_eq!(
    CommandBuilder::new("--index-runes-pre-alpha-i-agree-to-get-rekt --regtest runes")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      runes: vec![
        (
          Rune(RUNE),
          RuneInfo {
            burned: 0,
            divisibility: 0,
            end: None,
            etching: a.transaction,
            height: 2,
            id: RuneId {
              height: 2,
              index: 1
            },
            index: 1,
            limit: None,
            number: 0,
            rune: Rune(RUNE),
            supply: 1000,
            symbol: Some('¢'),
            timestamp: ord::timestamp(2),
          }
        ),
        (
          Rune(RUNE + 1),
          RuneInfo {
            burned: 0,
            divisibility: 0,
            end: None,
            etching: b.transaction,
            height: 4,
            id: RuneId {
              height: 4,
              index: 1
            },
            index: 1,
            limit: None,
            number: 1,
            rune: Rune(RUNE + 1),
            supply: 1000,
            symbol: Some('¢'),
            timestamp: ord::timestamp(4),
          }
        )
      ]
      .into_iter()
      .collect(),
    }
  );
}
