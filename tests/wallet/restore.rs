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

  CommandBuilder::new(["wallet", "restore", "--from", "mnemonic"])
    .stdin(mnemonic.to_string().into())
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
    "--from",
    "mnemonic",
  ])
  .stdin(mnemonic.to_string().into())
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
    .ord_rpc_server(&ord_rpc_server)
    .stderr_regex(".*")
    .run_and_deserialize_output::<ListDescriptorsResult>();

  CommandBuilder::new("wallet restore --from descriptor")
    .stdin(serde_json::to_string(&output).unwrap().as_bytes().to_vec())
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: wallet `ord` already exists\n")
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

  CommandBuilder::new("wallet --name foo restore --from descriptor")
      .stdin(r#"
{
  "wallet_name": "bar",
  "descriptors": [
    {
      "desc": "rawtr(cVMYXp8uf1yFU9AAY6NJu1twA2uT94mHQBGkfgqCCzp6RqiTWCvP)#tah5crv7",
      "timestamp": 1706047934,
      "active": false,
      "internal": null,
      "range": null,
      "next": null
    },
    {
      "desc": "rawtr(cVdVu6VRwYXsTPMiptqVYLcp7EtQi5sjxLzbPTSNwW6CkCxBbEFs)#5afaht8d",
      "timestamp": 1706047934,
      "active": false,
      "internal": null,
      "range": null,
      "next": null
    },
    {
      "desc": "wpkh([c0b9536d/86'/1'/0']tprv8fXhtVjj3vb7kgxKuiWXzcUsur44gbLbbtwxL4HKmpzkBNoMrYqbQhMe7MWhrZjLFc9RBpTRYZZkrS8HH1Q3SmD5DkfpjKqtd97q1JWfqzr/0/*)#dweuu0ww",
      "timestamp": 1706047839,
      "active": true,
      "internal": false,
      "range": [
        0,
        1000
      ],
      "next": 1
    },
    {
      "desc": "tr([c0b9536d/86'/1'/0']tprv8fXhtVjj3vb7kgxKuiWXzcUsur44gbLbbtwxL4HKmpzkBNoMrYqbQhMe7MWhrZjLFc9RBpTRYZZkrS8HH1Q3SmD5DkfpjKqtd97q1JWfqzr/1/*)#u6uap67k",
      "timestamp": 1706047839,
      "active": true,
      "internal": true,
      "range": [
        0,
        1013
      ],
      "next": 14
    }
  ]
}"#.into())
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: wallet \"foo\" contains unexpected output descriptors, and does not appear to be an `ord` wallet, create a new wallet with `ord wallet create`\n")
    .run_and_extract_stdout();
}

#[test]
fn restore_with_compact_works() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("wallet restore --from descriptor")
    .stdin(r#"{"wallet_name":"foo","descriptors":[{"desc":"rawtr(cVMYXp8uf1yFU9AAY6NJu1twA2uT94mHQBGkfgqCCzp6RqiTWCvP)#tah5crv7","timestamp":1706047934,"active":false,"internal":null,"range":null,"next":null},{"desc":"rawtr(cVdVu6VRwYXsTPMiptqVYLcp7EtQi5sjxLzbPTSNwW6CkCxBbEFs)#5afaht8d","timestamp":1706047934,"active":false,"internal":null,"range":null,"next":null},{"desc":"tr([c0b9536d/86'/1'/0']tprv8fXhtVjj3vb7kgxKuiWXzcUsur44gbLbbtwxL4HKmpzkBNoMrYqbQhMe7MWhrZjLFc9RBpTRYZZkrS8HH1Q3SmD5DkfpjKqtd97q1JWfqzr/0/*)#dweuu0ww","timestamp":1706047839,"active":true,"internal":false,"range":[0,1000],"next":1},{"desc":"tr([c0b9536d/86'/1'/0']tprv8fXhtVjj3vb7kgxKuiWXzcUsur44gbLbbtwxL4HKmpzkBNoMrYqbQhMe7MWhrZjLFc9RBpTRYZZkrS8HH1Q3SmD5DkfpjKqtd97q1JWfqzr/1/*)#u6uap67k","timestamp":1706047839,"active":true,"internal":true,"range":[0,1013],"next":14}]}"#.into())
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .expected_exit_code(0)
    .run_and_extract_stdout();
}

#[test]
fn restore_with_blank_mnemonic_generates_same_descriptors() {
  let (mnemonic, descriptors) = {
    let rpc_server = test_bitcoincore_rpc::spawn();

    let create::Output { mnemonic, .. } = CommandBuilder::new("wallet create")
      .bitcoin_rpc_server(&rpc_server)
      .run_and_deserialize_output();

    (mnemonic, rpc_server.descriptors())
  };

  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new(["wallet", "restore", "--from", "mnemonic"])
    .stdin(mnemonic.to_string().into())
    .bitcoin_rpc_server(&rpc_server)
    .run_and_extract_stdout();

  assert_eq!(rpc_server.descriptors(), descriptors);
}

#[test]
fn passphrase_conflicts_with_descriptor() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  CommandBuilder::new([
    "wallet",
    "restore",
    "--from",
    "descriptor",
    "--passphrase",
    "supersecurepassword",
  ])
  .stdin("".into())
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: descriptor does not take a passphrase\n")
  .run_and_extract_stdout();
}
