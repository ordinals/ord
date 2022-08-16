use super::*;

fn path(path: &str) -> String {
  if cfg!(target_os = "macos") {
    format!("Library/Application Support/{}", path)
  } else {
    format!(".local/share/{}", path)
  }
}

#[test]
fn init_existing_wallet() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  assert!(state
    .tempdir
    .path()
    .join(path("ord/regtest/wallet.sqlite"))
    .exists());

  assert!(state
    .tempdir
    .path()
    .join(path("ord/regtest/entropy"))
    .exists());

  Test::with_state(state)
    .command("--network regtest wallet init")
    .expected_status(1)
    .expected_stderr("error: Wallet already exists.\n")
    .run()
}

#[test]
fn init_nonexistent_wallet() {
  let output = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output();

  assert!(output
    .state
    .tempdir
    .path()
    .join(path("ord/regtest/wallet.sqlite"))
    .exists());

  assert!(output
    .state
    .tempdir
    .path()
    .join(path("ord/regtest/entropy"))
    .exists());
}

#[test]
fn load_corrupted_entropy() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  let entropy_path = state.tempdir.path().join(path("ord/regtest/entropy"));

  assert!(entropy_path.exists());

  let mut entropy = fs::read(&entropy_path).unwrap();
  entropy[0] ^= 0b0000_1000;

  fs::write(&entropy_path, entropy).unwrap();

  Test::with_state(state)
    .command("--network regtest wallet fund")
    .expected_status(1)
    .expected_stderr("error: ChecksumMismatch\n")
    .run();
}

#[test]
fn fund_existing_wallet() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  Test::with_state(state)
    .command("--network regtest wallet fund")
    .stdout_regex("^bcrt1.*\n")
    .run();
}

#[test]
fn fund_nonexistent_wallet() {
  Test::new()
    .command("--network regtest wallet fund")
    .expected_status(1)
    .expected_stderr("error: Wallet doesn't exist.\n")
    .run();
}

#[test]
fn utxos() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  let output = Test::with_state(state)
    .command("--network regtest wallet fund")
    .stdout_regex("^bcrt1.*\n")
    .output();

  output
    .state
    .client
    .generate_to_address(
      1,
      &Address::from_str(
        output
          .stdout
          .strip_suffix('\n')
          .ok_or("Failed to strip suffix")
          .unwrap(),
      )
      .unwrap(),
    )
    .unwrap();

  Test::with_state(output.state)
    .command("--network regtest wallet utxos")
    .expected_status(0)
    .stdout_regex("^[[:xdigit:]]{64}:0 5000000000\n")
    .run()
}

#[test]
fn balance() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  let state = Test::with_state(state)
    .command("--network regtest wallet balance")
    .expected_status(0)
    .expected_stdout("0\n")
    .output()
    .state;

  let output = Test::with_state(state)
    .command("--network regtest wallet fund")
    .stdout_regex("^bcrt1.*\n")
    .output();

  output
    .state
    .client
    .generate_to_address(
      101,
      &Address::from_str(
        output
          .stdout
          .strip_suffix('\n')
          .ok_or("Failed to strip suffix")
          .unwrap(),
      )
      .unwrap(),
    )
    .unwrap();

  Test::with_state(output.state)
    .command("--network regtest wallet balance")
    .expected_status(0)
    .expected_stdout("5000000000\n")
    .run()
}

#[test]
fn send_owned_ordinal() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  let output = Test::with_state(state)
    .command("--network regtest wallet fund")
    .stdout_regex("^bcrt1.*\n")
    .output();

  let from_address = Address::from_str(
    output
      .stdout
      .strip_suffix('\n')
      .ok_or("Failed to strip suffix")
      .unwrap(),
  )
  .unwrap();

  output
    .state
    .client
    .generate_to_address(1, &from_address)
    .unwrap();

  output
    .state
    .client
    .generate_to_address(
      100,
      &Address::from_str("bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw").unwrap(),
    )
    .unwrap();

  let mut output = Test::with_state(output.state)
    .command("--network regtest wallet utxos")
    .expected_status(0)
    .stdout_regex("[[:xdigit:]]{64}:[[:digit:]] 5000000000\n")
    .output();

  output.state.request(
    &format!(
      "api/list/{}",
      output
        .stdout
        .split(' ')
        .collect::<Vec<&str>>()
        .first()
        .unwrap()
    ),
    200,
    "[[5000000000,10000000000]]",
  );

  let wallet = Wallet::new(
    Bip84(
      (
        Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")
          .unwrap(),
        None,
      ),
      KeychainKind::External,
    ),
    None,
    Network::Regtest,
    MemoryDatabase::new(),
  )
  .unwrap();

  let to_address = wallet.get_address(AddressIndex::LastUnused).unwrap();

  let state = Test::with_state(output.state)
    .command(&format!(
      "--network regtest wallet send --address {to_address} --ordinal 5000000001",
    ))
    .expected_status(0)
    .stdout_regex(format!(
      "Sent ordinal 5000000001 to address {to_address}: [[:xdigit:]]{{64}}\n"
    ))
    .output()
    .state;

  wallet
    .sync(&state.blockchain, SyncOptions::default())
    .unwrap();

  state.client.generate_to_address(1, &from_address).unwrap();

  Test::with_state(state)
    .command(&format!(
      "--network regtest list {}",
      wallet.list_unspent().unwrap().first().unwrap().outpoint
    ))
    .expected_status(0)
    .expected_stdout("[5000000000,9999999780)\n")
    .run()
}

#[test]
fn send_foreign_ordinal() {
  let state = Test::new()
    .command("--network regtest wallet init")
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()
    .state;

  let output = Test::with_state(state)
    .command("--network regtest wallet fund")
    .stdout_regex("^bcrt1.*\n")
    .output();

  let from_address = Address::from_str(
    output
      .stdout
      .strip_suffix('\n')
      .ok_or("Failed to strip suffix")
      .unwrap(),
  )
  .unwrap();

  output
    .state
    .client
    .generate_to_address(1, &from_address)
    .unwrap();

  let mut output = Test::with_state(output.state)
    .command("--network regtest wallet utxos")
    .expected_status(0)
    .stdout_regex("[[:xdigit:]]{64}:[[:digit:]] 5000000000\n")
    .output();

  output.state.request(
    &format!(
      "api/list/{}",
      output
        .stdout
        .split(' ')
        .collect::<Vec<&str>>()
        .first()
        .unwrap()
    ),
    200,
    "[[5000000000,10000000000]]",
  );

  let wallet = Wallet::new(
    Bip84(
      (
        Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")
          .unwrap(),
        None,
      ),
      KeychainKind::External,
    ),
    None,
    Network::Regtest,
    MemoryDatabase::new(),
  )
  .unwrap();

  let to_address = wallet.get_address(AddressIndex::LastUnused).unwrap();

  Test::with_state(output.state)
    .command(&format!(
      "--network regtest wallet send --address {to_address} --ordinal 4999999999",
    ))
    .expected_status(1)
    .expected_stderr("error: No utxo contains 4999999999˚.\n")
    .run()
}
