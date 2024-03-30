use super::*;

#[cfg(unix)]
#[test]
fn wallet_resume() {
  use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
  };

  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--regtest", "--index-runes"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let batchfile = batch::File {
    etching: Some(batch::Etching {
      divisibility: 0,
      rune: SpacedRune {
        rune: Rune(RUNE),
        spacers: 0,
      },
      supply: "1000".parse().unwrap(),
      premine: "1000".parse().unwrap(),
      symbol: '¢',
      ..default()
    }),
    inscriptions: vec![batch::Entry {
      file: "inscription.jpeg".into(),
      ..default()
    }],
    ..default()
  };

  let tempdir = Arc::new(TempDir::new().unwrap());

  {
    let mut spawn =
      CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
        .temp_dir(tempdir.clone())
        .write("batch.yaml", serde_yaml::to_string(&batchfile).unwrap())
        .write("inscription.jpeg", "inscription")
        .bitcoin_rpc_server(&bitcoin_rpc_server)
        .ord_rpc_server(&ord_rpc_server)
        .expected_exit_code(1)
        .spawn();

    let mut buffer = String::new();

    BufReader::new(spawn.child.stderr.as_mut().unwrap())
      .read_line(&mut buffer)
      .unwrap();

    assert_eq!(buffer, "Waiting for rune commitment to mature…\n");

    bitcoin_rpc_server.mine_blocks(1);

    signal::kill(
      Pid::from_raw(spawn.child.id().try_into().unwrap()),
      Signal::SIGINT,
    )
    .unwrap();

    buffer.clear();

    BufReader::new(spawn.child.stderr.as_mut().unwrap())
      .read_line(&mut buffer)
      .unwrap();

    assert_eq!(
      buffer,
      "Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.\n"
    );

    spawn.child.wait().unwrap();
  }

  //  {
  //    CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
  //      .temp_dir(tempdir)
  //      .bitcoin_rpc_server(&bitcoin_rpc_server)
  //      .ord_rpc_server(&ord_rpc_server)
  //      .expected_exit_code(1)
  //      .expected_stderr(
  //        "error: rune `AAAAAAAAAAAAA` has a pending etching, resume it with `ord wallet resume`\n",
  //      )
  //      .run_and_extract_stdout();
  //  }

  CommandBuilder::new("--regtest --index-runes wallet resume")
    .temp_dir(tempdir)
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_extract_stdout();
}
