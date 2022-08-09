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
    .join(path("ord/wallet.sqlite"))
    .exists());

  assert!(state.tempdir.path().join(path("ord/entropy")).exists());

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
    .join(path("ord/wallet.sqlite"))
    .exists());

  assert!(output
    .state
    .tempdir
    .path()
    .join(path("ord/entropy"))
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

  let entropy_path = state.tempdir.path().join(path("ord/entropy"));

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
    .command("--network regtest wallet utxos")
    .expected_status(0)
    .stdout_regex("^[[:xdigit:]]{64}:0 5000000000\n")
    .run()
}
