use super::*;

#[test]
fn inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let reveal_txid = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new("wallet inscribe hello.txt")
      .write("hello.txt", "HELLOWORLD")
      .rpc_server(&rpc_server)
      .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
      .run(),
  );
  rpc_server.mine_blocks(1);

  let inscription_id = format!("{reveal_txid}i0");

  CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{reveal_txid}:0:0\n"))
    .run();

  let stdout = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  let address = stdout.trim();

  let stdout = CommandBuilder::new(format!("wallet send {address} {inscription_id}"))
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  rpc_server.mine_blocks(1);

  let txid = Txid::from_str(stdout.trim()).unwrap();

  let outpoint = OutPoint::new(txid, 0);

  CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{outpoint}:0\n"))
    .run();
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let txid = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new("wallet inscribe hello.txt")
      .write("hello.txt", "HELLOWORLD")
      .rpc_server(&rpc_server)
      .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
      .run(),
  );

  rpc_server.mine_blocks(1);

  rpc_server.lock(OutPoint { txid, vout: 0 });

  CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{txid}i0\t{txid}:0:0\n"))
    .run();
}
