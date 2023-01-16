use super::*;

#[test]
fn transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  assert!(rpc_server.loaded_wallets().is_empty());

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run();

  assert_eq!(rpc_server.loaded_wallets().len(), 1);
  assert_eq!(rpc_server.loaded_wallets().first().unwrap(), "ord");

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n")
    .run();
}

#[test]
fn transactions_with_limit() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n")
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n[[:xdigit:]]{64}\t2\n")
    .run();

  CommandBuilder::new("wallet transactions --limit 1")
    .rpc_server(&rpc_server)
    .stdout_regex("[[:xdigit:]]{64}\t1\n")
    .run();
}
