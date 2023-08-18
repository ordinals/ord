use {super::*, ord::subcommand::wallet::transactions::Output};

#[test]
fn transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  assert!(rpc_server.loaded_wallets().is_empty());

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Vec<Output>>();

  assert_eq!(rpc_server.loaded_wallets().len(), 1);
  assert_eq!(rpc_server.loaded_wallets().first().unwrap(), "ord");

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Vec<Output>>();

  assert_regex_match!(output[0].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[0].confirmations, 1);
}

#[test]
fn transactions_with_limit() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run_and_extract_stdout();

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Vec<Output>>();

  assert_regex_match!(output[0].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[0].confirmations, 1);

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Vec<Output>>();

  assert_regex_match!(output[1].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[1].confirmations, 2);

  let output = CommandBuilder::new("wallet transactions --limit 1")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Vec<Output>>();

  assert_regex_match!(output[0].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[0].confirmations, 1);
}
