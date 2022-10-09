use super::*;

#[test]
fn publish() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("--chain regtest rune publish --name foo --ordinal 0")
    .rpc_server(&rpc_server)
    .run();
}

#[test]
fn publish_mainnet_forbidden() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("rune publish --name foo --ordinal 0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: `ord rune publish` is unstable and not yet supported on mainnet.\n")
    .expected_exit_code(1)
    .run();
}
