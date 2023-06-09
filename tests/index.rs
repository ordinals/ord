use {super::*, crate::command_builder::ToArgs};

#[test]
fn custom_index_path() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("foo.redb");

  CommandBuilder::new(format!("--index {} index", index_path.display()))
    .rpc_server(&rpc_server)
    .run();

  assert!(index_path.is_file())
}

#[test]
fn re_opening_database_does_not_trigger_schema_check() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("foo.redb");

  CommandBuilder::new(format!("--index {} index", index_path.display()))
    .rpc_server(&rpc_server)
    .run();

  assert!(index_path.is_file());

  CommandBuilder::new(format!("--index {} index", index_path.display()))
    .rpc_server(&rpc_server)
    .run();
}

#[test]
fn index_runs_with_rpc_user_and_pass_as_env_vars() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();

  let ord = Command::new(executable_path("ord"))
    .args(
      format!(
        "--rpc-url {} --bitcoin-data-dir {} --data-dir {} index",
        rpc_server.url(),
        tempdir.path().display(),
        tempdir.path().display()
      )
      .to_args(),
    )
    .env("ORD_BITCOIN_RPC_PASS", "bar")
    .env("ORD_BITCOIN_RPC_USER", "foo")
    .env("ORD_INTEGRATION_TEST", "1")
    .current_dir(&tempdir)
    .spawn()
    .unwrap();

  rpc_server.mine_blocks(1);

  assert_eq!(ord.wait_with_output().unwrap().status.code(), Some(0));
}
