use {super::*, ord::subcommand::wallet::create::Output};

#[test]
fn create() {
  let core = mockcore::spawn();

  assert!(!core.wallets().contains("ord"));

  CommandBuilder::new("wallet create")
    .core(&core)
    .run_and_deserialize_output::<Output>();

  assert!(core.wallets().contains("ord"));
}

#[test]
fn seed_phrases_are_twelve_words_long() {
  let Output { mnemonic, .. } = CommandBuilder::new("wallet create")
    .core(&mockcore::spawn())
    .run_and_deserialize_output();

  assert_eq!(mnemonic.word_count(), 12);
}

#[test]
fn wallet_creates_correct_mainnet_taproot_descriptor() {
  let core = mockcore::spawn();

  CommandBuilder::new("wallet create")
    .core(&core)
    .run_and_deserialize_output::<Output>();

  assert_eq!(core.descriptors().len(), 2);
  assert_regex_match!(
    &core.descriptors()[0],
    r"tr\(\[[[:xdigit:]]{8}/86'/0'/0'\]xprv[[:alnum:]]*/0/\*\)#[[:alnum:]]{8}"
  );
  assert_regex_match!(
    &core.descriptors()[1],
    r"tr\(\[[[:xdigit:]]{8}/86'/0'/0'\]xprv[[:alnum:]]*/1/\*\)#[[:alnum:]]{8}"
  );
}

#[test]
fn wallet_creates_correct_test_network_taproot_descriptor() {
  let core = mockcore::builder().network(Network::Signet).build();

  CommandBuilder::new("--chain signet wallet create")
    .core(&core)
    .run_and_deserialize_output::<Output>();

  assert_eq!(core.descriptors().len(), 2);
  assert_regex_match!(
    &core.descriptors()[0],
    r"tr\(\[[[:xdigit:]]{8}/86'/1'/0'\]tprv[[:alnum:]]*/0/\*\)#[[:alnum:]]{8}"
  );
  assert_regex_match!(
    &core.descriptors()[1],
    r"tr\(\[[[:xdigit:]]{8}/86'/1'/0'\]tprv[[:alnum:]]*/1/\*\)#[[:alnum:]]{8}"
  );
}

#[test]
fn detect_wrong_descriptors() {
  let core = mockcore::spawn();

  CommandBuilder::new("wallet create")
    .core(&core)
    .run_and_deserialize_output::<Output>();

  core.import_descriptor("wpkh([aslfjk])#a23ad2l".to_string());

  CommandBuilder::new("wallet transactions")
    .core(&core)
    .stderr_regex(
      r#"error: wallet "ord" contains unexpected output descriptors, and does not appear to be an `ord` wallet, create a new wallet with `ord wallet create`\n"#,
    )
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn create_with_different_name() {
  let core = mockcore::spawn();

  assert!(!core.wallets().contains("inscription-wallet"));

  CommandBuilder::new("wallet --name inscription-wallet create")
    .core(&core)
    .run_and_deserialize_output::<Output>();

  assert!(core.wallets().contains("inscription-wallet"));
}
