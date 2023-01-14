use super::*;

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

  CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription}\t{reveal}:0:0\n"))
    .run();

  let stdout = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  let address = stdout.trim();

  let stdout = CommandBuilder::new(format!("wallet send {address} {inscription}"))
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  rpc_server.mine_blocks(1);

  let txid = Txid::from_str(stdout.trim()).unwrap();

  let outpoint = OutPoint::new(txid, 0);

  CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription}\t{outpoint}:0\n"))
    .run();
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

  CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription}\t{reveal}:0:0\n"))
    .run();
}
