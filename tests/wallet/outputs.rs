use super::*;

#[test]
fn outputs() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{outpoint}\t{amount}\n"))
    .run();
}

#[test]
fn outputs_includes_locked_outputs() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  rpc_server.lock(outpoint);

  CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{outpoint}\t{amount}\n"))
    .run();
}
