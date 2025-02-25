use {
  super::*,
  ord::subcommand::wallet::{create::Output, dump::Output as Dump},
};

#[test]
fn create() {
  let core = mockcore::spawn();

  let tempdir = Arc::new(TempDir::new().unwrap());

  let wallet_db = tempdir.path().join("wallets/ord.redb");

  assert!(!wallet_db.try_exists().unwrap());

  CommandBuilder::new("wallet create")
    .core(&core)
    .temp_dir(tempdir.clone())
    .run_and_deserialize_output::<Output>();

  assert!(wallet_db.try_exists().unwrap());
  assert!(wallet_db.is_file());
}

#[test]
fn create_with_same_name_fails() {
  let core = mockcore::spawn();

  let tempdir = TempDir::new().unwrap();

  let wallet_db = tempdir.path().join("wallets/ord.redb");

  assert!(!wallet_db.try_exists().unwrap());

  let arc = Arc::new(tempdir);

  CommandBuilder::new("wallet create")
    .core(&core)
    .temp_dir(arc.clone())
    .run_and_deserialize_output::<Output>();

  assert!(wallet_db.try_exists().unwrap());

  CommandBuilder::new("wallet create")
    .core(&core)
    .temp_dir(arc.clone())
    .expected_exit_code(1)
    .stderr_regex("error: wallet ord at .* already exists.*")
    .run_and_extract_stdout();
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
  let ord = TestServer::spawn(&core);

  let tempdir = Arc::new(TempDir::new().unwrap());

  CommandBuilder::new("wallet create")
    .temp_dir(tempdir.clone())
    .core(&core)
    .run_and_deserialize_output::<Output>();

  let dump = CommandBuilder::new("wallet dump")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Dump>();

  assert_regex_match!(
    &dump.descriptor,
    r"tr\(\[[[:xdigit:]]{8}/86'/0'/0'\]xprv[[:alnum:]]*/0/\*\)#[[:alnum:]]{8}"
  );

  assert_regex_match!(
    &dump.change_descriptor,
    r"tr\(\[[[:xdigit:]]{8}/86'/0'/0'\]xprv[[:alnum:]]*/1/\*\)#[[:alnum:]]{8}"
  );
}

#[test]
fn wallet_creates_correct_test_network_taproot_descriptor() {
  let core = mockcore::builder().network(Network::Signet).build();
  let ord = TestServer::spawn_with_args(&core, &["--signet"]);

  let tempdir = Arc::new(TempDir::new().unwrap());

  CommandBuilder::new("--chain signet wallet create")
    .temp_dir(tempdir.clone())
    .core(&core)
    .run_and_deserialize_output::<Output>();

  let dump = CommandBuilder::new("--chain signet wallet dump")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Dump>();

  assert_regex_match!(
    &dump.descriptor,
    r"tr\(\[[[:xdigit:]]{8}/86'/1'/0'\]tprv[[:alnum:]]*/0/\*\)#[[:alnum:]]{8}"
  );

  assert_regex_match!(
    &dump.change_descriptor,
    r"tr\(\[[[:xdigit:]]{8}/86'/1'/0'\]tprv[[:alnum:]]*/1/\*\)#[[:alnum:]]{8}"
  );
}

#[test]
fn create_with_different_name() {
  let core = mockcore::spawn();

  let tempdir = Arc::new(TempDir::new().unwrap());

  let wallet_name = "inscription-wallet";

  let wallet_db = tempdir.path().join(format!("wallets/{wallet_name}.redb"));

  assert!(!wallet_db.try_exists().unwrap());

  CommandBuilder::new(format!("wallet --name {wallet_name} create"))
    .core(&core)
    .temp_dir(tempdir.clone())
    .run_and_deserialize_output::<Output>();

  assert!(wallet_db.try_exists().unwrap());
  assert!(wallet_db.is_file());
}
