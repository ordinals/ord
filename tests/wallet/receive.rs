use super::*;

#[test]
fn receive_cardinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new("wallet receive --cardinal")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();

  assert!(Address::from_str(stdout.trim()).is_ok());
}

#[test]
fn receive_ordinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();

  assert!(stdout.starts_with("ord1"));
}
