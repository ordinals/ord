use {super::*, ord::subcommand::wallet::balance::Output};

#[test]
fn authentication() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(
    &core,
    &["--server-username", "foo", "--server-password", "bar"],
    &[],
  );

  create_wallet(&core, &ord);

  assert_eq!(
    CommandBuilder::new("--server-username foo --server-password bar wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Output>()
      .cardinal,
    0
  );

  core.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("--server-username foo --server-password bar wallet balance")
      .core(&core)
      .ord(&ord)
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
