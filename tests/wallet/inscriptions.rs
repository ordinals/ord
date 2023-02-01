use {
  super::*,
  ord::subcommand::wallet::{inscriptions::Output, receive},
};

#[test]
fn inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe {
    reveal,
    inscription,
    ..
  } = inscribe(&rpc_server);

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .output::<Vec<Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription.parse().unwrap());
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
  assert_eq!(
    output[0].explorer,
    format!("https://ordinals.com/inscription/{inscription}")
  );

  let address = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .output::<receive::Output>()
    .address;

  let stdout = CommandBuilder::new(format!("wallet send {address} {inscription}"))
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  rpc_server.mine_blocks(1);

  let txid = Txid::from_str(stdout.trim()).unwrap();

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .output::<Vec<Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription.parse().unwrap());
  assert_eq!(output[0].location, format!("{txid}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let Inscribe {
    inscription,
    reveal,
    ..
  } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  rpc_server.lock(OutPoint {
    txid: reveal,
    vout: 0,
  });

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .output::<Vec<Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription.parse().unwrap());
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_with_desynced_index_fails() {
  let tempdir = TempDir::new().unwrap().into_path();
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  CommandBuilder::new("index")
    .with_tempdir(tempdir.clone())
    .rpc_server(&rpc_server)
    .run();

  let desynced_rpc_server = test_bitcoincore_rpc::spawn();
  desynced_rpc_server.mine_blocks_with_subsidy(1, 10_000);
  create_wallet(&desynced_rpc_server);

  CommandBuilder::new("wallet inscriptions")
    .with_tempdir(tempdir)
    .rpc_server(&desynced_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: output in Bitcoin Core but not in ordinals index\n")
    .run();
}
