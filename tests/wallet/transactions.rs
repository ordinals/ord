use {super::*, ord::subcommand::wallet::transactions::Output};

#[test]
fn transactions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert!(bitcoin_rpc_server.loaded_wallets().is_empty());

  CommandBuilder::new("wallet transactions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(bitcoin_rpc_server.loaded_wallets().len(), 1);
  assert_eq!(bitcoin_rpc_server.loaded_wallets().first().unwrap(), "ord");

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_regex_match!(output[0].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[0].confirmations, 1);
}

#[test]
fn transactions_with_limit() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("wallet transactions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .stdout_regex(".*")
    .run_and_extract_stdout();

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_regex_match!(output[0].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[0].confirmations, 1);

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_regex_match!(output[1].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[1].confirmations, 2);

  let output = CommandBuilder::new("wallet transactions --limit 1")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_regex_match!(output[0].transaction.to_string(), "[[:xdigit:]]{64}");
  assert_eq!(output[0].confirmations, 1);
}
