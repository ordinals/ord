use {super::*, ord::subcommand::wallet::transactions::Output};

#[test]
fn transactions() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  assert!(core.loaded_wallets().is_empty());

  CommandBuilder::new("wallet transactions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(core.loaded_wallets().len(), 1);
  assert_eq!(core.loaded_wallets().first().unwrap(), "ord");

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output[0].confirmations, 1);
}

#[test]
fn transactions_with_limit() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("wallet transactions")
    .core(&core)
    .ord(&ord)
    .stdout_regex(".*")
    .run_and_extract_stdout();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output.len(), 1);

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet transactions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output.len(), 2);

  let output = CommandBuilder::new("wallet transactions --limit 1")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output.len(), 1);
}
