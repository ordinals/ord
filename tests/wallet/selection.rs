use super::*;

#[test]
fn inscribe_does_not_select_runic_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  drain(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("--regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: wallet contains no cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_amount_does_not_select_runic_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  drain(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("--regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 600sat")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: not enough cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_satpoint_does_not_send_runic_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10000);

  let etched = etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  CommandBuilder::new(format!("--regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw {}:0", etched.inscribe.rune.unwrap().location.unwrap()))
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: runic outpoints may not be sent by satpoint\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn send_inscription_does_not_select_runic_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));

  let (id, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  drain(&bitcoin_rpc_server, &ord_rpc_server);

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
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn mint_does_not_select_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  batch(
    &bitcoin_rpc_server,
    &ord_rpc_server,
    Batchfile {
      etch: Some(Etch {
        divisibility: 1,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        mint: Some(ord::wallet::inscribe::BatchMint {
          deadline: None,
          limit: "1000".parse().unwrap(),
          term: None,
        }),
      }),
      inscriptions: vec![BatchEntry {
        file: "inscription.jpeg".into(),
        ..Default::default()
      }],
      ..Default::default()
    },
  );

  drain(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet mint --fee-rate 0 --rune {}",
    Rune(RUNE)
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_does_not_send_inscription() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10000);

  let rune = Rune(RUNE);

  CommandBuilder::new("--chain regtest --index-runes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks_with_subsidy(1, 10000);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 10000,
      ordinal: 10000,
      runic: Some(0),
      runes: Some(BTreeMap::new()),
      total: 20000,
    }
  );

  etch(&bitcoin_rpc_server, &ord_rpc_server, rune);

  drain(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new(format!(
    "
       --chain regtest
       --index-runes
       wallet send
       --fee-rate 0
       bcrt1pyrmadgg78e38ewfv0an8c6eppk2fttv5vnuvz04yza60qau5va0saknu8k
       1000{rune}
     ",
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}
