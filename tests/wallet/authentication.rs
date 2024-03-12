use {super::*, ord::subcommand::wallet::balance::Output};

#[test]
fn authentication() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(
    &bitcoin_rpc_server,
    &["--server-username", "foo", "--server-password", "bar"],
    &[],
  );

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new("--server-username foo --server-password bar wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>()
      .cardinal,
    0
  );

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("--server-username foo --server-password bar wallet balance")
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .ord_rpc_server(&ord_rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      runic: None,
      runes: None,
      total: 50 * COIN_VALUE,
    }
  );
}
