use {super::*, ord::subcommand::wallet::create, ord::subcommand::Empty};

#[test]
fn restore_generates_same_descriptors() {
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let create::Output { mnemonic, .. } = CommandBuilder::new("wallet create")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output();

    (mnemonic, rpc_server.descriptors())
  };

  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new(["wallet", "restore", &mnemonic.to_string()])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Empty>();

  assert_eq!(rpc_server.descriptors(), descriptors);
}

#[test]
fn restore_generates_same_descriptors_with_passphrase() {
  let passphrase = "foo";
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let create::Output { mnemonic, .. } =
      CommandBuilder::new(["wallet", "create", "--passphrase", passphrase])
        .rpc_server(&rpc_server)
        .run_and_deserialize_output();

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
  .run_and_deserialize_output::<Empty>();

  assert_eq!(rpc_server.descriptors(), descriptors);
}
