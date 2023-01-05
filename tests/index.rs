use super::*;

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
