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
fn wallet_creates_correct_mainnet_taproot_descriptor() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Bitcoin)
    .build();

  CommandBuilder::new("wallet create")
    .rpc_server(&rpc_server)
    .run();

  assert_eq!(rpc_server.descriptors().len(), 2);
  assert_regex_match!(
    &rpc_server.descriptors()[0],
    r"tr\(\[[[:xdigit:]]{8}/86'/0'/0'\]xprv[[:alnum:]]*/0/\*\)#[[:alnum:]]{8}"
  );
  assert_regex_match!(
    &rpc_server.descriptors()[1],
    r"tr\(\[[[:xdigit:]]{8}/86'/0'/0'\]xprv[[:alnum:]]*/1/\*\)#[[:alnum:]]{8}"
  );
}

#[test]
fn wallet_creates_correct_taproot_descriptor() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();

  CommandBuilder::new("--chain signet wallet create")
    .rpc_server(&rpc_server)
    .run();

  assert_eq!(rpc_server.descriptors().len(), 2);
  assert_regex_match!(
    &rpc_server.descriptors()[0],
    r"tr\(\[[[:xdigit:]]{8}/86'/1'/0'\]tprv[[:alnum:]]*/0/\*\)#[[:alnum:]]{8}"
  );
  assert_regex_match!(
    &rpc_server.descriptors()[1],
    r"tr\(\[[[:xdigit:]]{8}/86'/1'/0'\]tprv[[:alnum:]]*/1/\*\)#[[:alnum:]]{8}"
  );
}

#[test]
fn detect_wrong_descriptors() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  CommandBuilder::new("--chain regtest wallet create")
    .rpc_server(&rpc_server)
    .run();

  rpc_server.import_descriptor("wpkh([aslfjk])#a23ad2l".to_string());

  CommandBuilder::new("--chain regtest wallet transactions")
    .rpc_server(&rpc_server)
    .stderr_regex(
      "error: this does not appear to be an ord wallet, please create one using `ord wallet create`\n",
    )
    .expected_exit_code(1)
    .run();
}

#[ignore]
#[test]
fn consecutive_create_throws_error() {}
