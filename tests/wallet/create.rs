use super::*;

#[test]
fn create() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  assert!(!rpc_server.wallets().contains("ord"));

  CommandBuilder::new("--chain regtest wallet create")
    .rpc_server(&rpc_server)
    .run();

  assert!(rpc_server.wallets().contains("ord"));
}

#[test]
fn wallet_creates_correct_taproot_descriptor() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  CommandBuilder::new("--chain regtest wallet create")
    .rpc_server(&rpc_server)
    .run();

  assert_eq!(rpc_server.descriptors().len(), 1);
  assert_regex_match!(
    &rpc_server.descriptors()[0],
    r"tr\(\[.*/86'/0'/0'\]tprv.*/0/\*\)#.*"
  );
}
