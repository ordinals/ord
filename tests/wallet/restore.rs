use super::*;

#[test]
fn restore_generates_same_descriptors() {
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let Create { mnemonic } = CommandBuilder::new("wallet create")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Create>();

    (mnemonic, rpc_server.descriptors())
  };

  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new(["wallet", "restore", &mnemonic.to_string()])
    .rpc_server(&rpc_server)
    .run_and_extract_stdout();

  assert_eq!(rpc_server.descriptors(), descriptors);
}

#[test]
fn restore_generates_same_descriptors_with_passphrase() {
  let passphrase = "foo";
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let Create { mnemonic } = CommandBuilder::new(["wallet", "create", "--passphrase", passphrase])
      .rpc_server(&rpc_server)
      .run_and_check_output::<Create>();

    (mnemonic, rpc_server.descriptors())
  };

  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new([
    "wallet",
    "restore",
    "--passphrase",
    passphrase,
    &mnemonic.to_string(),
  ])
  .rpc_server(&rpc_server)
  .run_and_extract_stdout();

  assert_eq!(rpc_server.descriptors(), descriptors);
}
