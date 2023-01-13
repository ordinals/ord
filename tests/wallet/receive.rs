use super::*;

#[test]
fn receive() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();

  assert!(Address::from_str(stdout.trim()).is_ok());
}
