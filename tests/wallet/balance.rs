use {super::*, ord::subcommand::wallet::balance::Output};

#[test]
fn wallet_balance() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Output>()
      .cardinal,
    0
  );

  rpc_server.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Output>()
      .cardinal,
    50 * COIN_VALUE
  );
}

#[test]
fn wallet_balance_only_counts_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Output>()
      .cardinal,
    0
  );

  inscribe(&rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Output>()
      .cardinal,
    100 * COIN_VALUE - 10_000
  );
}
