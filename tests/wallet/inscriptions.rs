use super::*;

#[test]
fn inscriptions() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .wallet_name("ord-wallet")
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let inscription_id = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new(format!(
      "--chain signet wallet inscribe --satpoint {txid}:0:0 hello.txt"
    ))
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run(),
  );

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{inscription_id}:0:0\n"))
    .run();

  let stdout = CommandBuilder::new("--chain signet wallet receive")
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  let address = stdout.trim();

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send {address} {inscription_id}"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run();

  rpc_server.mine_blocks(1);

  let txid = Txid::from_str(stdout.trim()).unwrap();

  let outpoint = OutPoint::new(txid, 0);

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{outpoint}:0\n"))
    .run();
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let inscription_id = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new("--chain signet wallet inscribe hello.txt")
      .write("hello.txt", "HELLOWORLD")
      .rpc_server(&rpc_server)
      .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
      .run(),
  );

  rpc_server.mine_blocks(1);

  rpc_server.lock(OutPoint {
    txid: inscription_id,
    vout: 0,
  });

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{inscription_id}:0:0\n"))
    .run();
}
