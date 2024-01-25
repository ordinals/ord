use {super::*, ord::subcommand::wallet::create};

#[test]
fn restore_generates_same_descriptors() {
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let create::Output { mnemonic, .. } = CommandBuilder::new("wallet create")
      .bitcoin_rpc_server(&rpc_server)
      .run_and_deserialize_output();

    (mnemonic, rpc_server.descriptors())
  };

  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new([
    "wallet",
    "restore",
    "--from-mnemonic",
    &mnemonic.to_string(),
  ])
  .bitcoin_rpc_server(&rpc_server)
  .run_and_extract_stdout();

  assert_eq!(rpc_server.descriptors(), descriptors);
}

#[test]
fn restore_generates_same_descriptors_with_passphrase() {
  let passphrase = "foo";
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let create::Output { mnemonic, .. } =
      CommandBuilder::new(["wallet", "create", "--passphrase", passphrase])
        .bitcoin_rpc_server(&rpc_server)
        .run_and_deserialize_output();

    (mnemonic, rpc_server.descriptors())
  };

  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new([
    "wallet",
    "restore",
    "--passphrase",
    passphrase,
    "--from-mnemonic",
    &mnemonic.to_string(),
  ])
  .bitcoin_rpc_server(&rpc_server)
  .run_and_extract_stdout();

  assert_eq!(rpc_server.descriptors(), descriptors);
}

#[test]
fn restore_to_existing_wallet_fails() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let descriptors = bitcoin_rpc_server.descriptors();

  let output = CommandBuilder::new("wallet dump")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .stderr_regex(".*")
    .run_and_deserialize_output::<BitcoinCoreDescriptors>();

  CommandBuilder::new("wallet restore --from-descriptors")
    .stdin(serde_json::to_string(&output).unwrap().as_bytes().to_vec())
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: cannot restore because wallet named `ord` already exists\n")
    .run_and_extract_stdout();

  assert_eq!(
    descriptors,
    output
      .descriptors
      .into_iter()
      .map(|descriptor| descriptor.desc)
      .collect::<Vec<String>>()
  );
}
#[test]
fn restore_with_wrong_descriptors_fails() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("wallet restore --from-descriptors")
    .stdin("
{
  \"wallet_name\": \"ord\",
  \"descriptors\": [
    {
      \"desc\": \"wpkh([0b17d84b/86'/0'/0']xprv9zVZyVjXfgLumbVTyCqsbHRPYJLeZGboyZcnkDexD2wUcrjbjxag3X24vPXf99XHbod4kCWauAcdGFEtAe7yUw1wR3SYhWxmybnZ64Revge/0/*)#vl5tp7gp\",
      \"timestamp\": \"now\",
      \"active\": true,
      \"internal\": null,
      \"range\": null,
      \"next\": null
    },
    {
      \"desc\": \"tr([0b17d84b/86'/0'/0']xprv9zVZyVjXfgLumbVTyCqsbHRPYJLeZGboyZcnkDexD2wUcrjbjxag3X24vPXf99XHbod4kCWauAcdGFEtAe7yUw1wR3SYhWxmybnZ64Revge/1/*)#at32utce\",
      \"timestamp\": \"now\",
      \"active\": true,
      \"internal\": null,
      \"range\": null,
      \"next\": null
    }
  ]
}
".into())
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: \n")
    .run_and_extract_stdout();
}
