use super::*;

fn path(path: &str) -> String {
  if cfg!(target_os = "macos") {
    format!("Library/Application Support/{}", path)
  } else {
    format!(".local/share/{}", path)
  }
}

#[test]
fn init_existing_wallet() -> Result {
  let output = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()?;

  assert!(output
    .tempdir
    .path()
    .join(path("ord/wallet.sqlite"))
    .exists());

  assert!(output.tempdir.path().join(path("ord/entropy")).exists());

  Test::connect(output)?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(1)
    .expected_stderr("error: Wallet already exists.\n")
    .run()
}

#[test]
fn init_nonexistent_wallet() -> Result {
  let tempdir = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()?
    .tempdir;

  assert!(tempdir.path().join(path("ord/wallet.sqlite")).exists());

  assert!(tempdir.path().join(path("ord/entropy")).exists());

  Ok(())
}

#[test]
fn load_corrupted_entropy() -> Result {
  let output = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()?;

  let entropy_path = output.tempdir.path().join(path("ord/entropy"));

  assert!(entropy_path.exists());

  let mut entropy = fs::read(&entropy_path)?;
  entropy[0] ^= 0b0000_1000;

  fs::write(&entropy_path, entropy)?;

  Test::connect(output)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .expected_status(1)
    .expected_stderr("error: ChecksumMismatch\n")
    .run()
}

#[test]
fn fund_existing_wallet() -> Result {
  let output = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .set_home_to_tempdir()
    .output()?;

  Test::connect(output)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .stdout_regex("^bcrt1.*\n")
    .run()
}

#[test]
fn fund_nonexistent_wallet() -> Result {
  Test::new()?
    .command("wallet fund")
    .set_home_to_tempdir()
    .expected_status(1)
    .expected_stderr("error: Wallet doesn't exist.\n")
    .run()
}

#[test]
fn utxos() -> Result {
  let output = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .set_home_to_tempdir()
    .output()?;

  let output = Test::connect(output)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .stdout_regex("^bcrt1.*\n")
    .output()?;

  output.client.generate_to_address(
    101,
    &Address::from_str(
      &output
        .stdout
        .strip_suffix('\n')
        .ok_or("Failed to strip suffix")?,
    )?,
  )?;

  Test::connect(output)?
    .command("wallet utxos")
    .set_home_to_tempdir()
    .expected_status(0)
    .stdout_regex("^[a-z0-9]{64}:[0-9]*\n")
    .run()
}

#[test]
fn balance() -> Result {
  let output = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .set_home_to_tempdir()
    .output()?;

  let output = Test::connect(output)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .stdout_regex("^bcrt1.*\n")
    .output()?;

  output.client.generate_to_address(
    101,
    &Address::from_str(
      &output
        .stdout
        .strip_suffix('\n')
        .ok_or("Failed to strip suffix")?,
    )?,
  )?;

  Test::connect(output)?
    .command("wallet balance")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stdout("5000000000\n")
    .run()
}
