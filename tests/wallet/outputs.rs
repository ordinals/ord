use {super::*, ord::subcommand::wallet::outputs::Output};

#[test]
fn outputs() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  let output = CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .output::<Vec<Output>>();

  assert_eq!(output[0].output, outpoint);
  assert_eq!(output[0].amount, amount);
}

#[test]
fn outputs_includes_locked_outputs() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  rpc_server.lock(outpoint);

  let output = CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .output::<Vec<Output>>();

  assert_eq!(output[0].output, outpoint);
  assert_eq!(output[0].amount, amount);
}

#[test]
fn outputs_with_desynced_index_fails() {
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

  CommandBuilder::new("wallet outputs")
    .with_tempdir(tempdir)
    .rpc_server(&desynced_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: output in Bitcoin Core but not in ordinals index\n")
    .run();
}
