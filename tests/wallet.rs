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
  let tempdir = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()?
    .tempdir;

  assert!(tempdir.path().join(path("ord/wallet.sqlite")).exists());

  assert!(tempdir.path().join(path("ord/entropy")).exists());

  Test::with_tempdir(tempdir)?
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
  let tempdir = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()?
    .tempdir;

  let entropy_path = tempdir.path().join(path("ord/entropy"));

  assert!(entropy_path.exists());

  let mut entropy = fs::read(&entropy_path)?;
  entropy[0] ^= 0b0000_1000;

  fs::write(&entropy_path, entropy)?;

  Test::with_tempdir(tempdir)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .expected_status(1)
    .expected_stderr("error: ChecksumMismatch\n")
    .run()?;

  Ok(())
}

#[test]
fn fund_existing_wallet() -> Result {
  let tempdir = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .set_home_to_tempdir()
    .output()?
    .tempdir;

  Test::with_tempdir(tempdir)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .stdout_regex("^tb1.*\n")
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

  let output = Test::with_tempdir(output.tempdir)?
    .command("wallet fund")
    .set_home_to_tempdir()
    .stdout_regex("^bcrt1.*\n")
    .output()?;

  let core_address = output.bitcoind.client.get_new_address(None, None)?;

  output
    .bitcoind
    .client
    .generate_to_address(101, &core_address)?;

  output.bitcoind.client.send_to_address(
    &Address::from_str(&output.stdout.strip_suffix('\n').unwrap())?,
    Amount::from_btc(10.0)?,
    None,
    None,
    None,
    None,
    None,
    None,
  )?;

  output
    .bitcoind
    .client
    .generate_to_address(1, &core_address)?;

  Test::with_tempdir(output.tempdir)?
    .command("wallet utxos")
    .set_home_to_tempdir()
    .run()
}
