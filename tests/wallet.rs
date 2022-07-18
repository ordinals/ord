use super::*;

#[test]
fn init() -> Result {
  let tempdir = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .output()?
    .tempdir;

  assert!(tempdir
    .path()
    .join(if cfg!(target_os = "macos") {
      "Library/Application Support/ord/wallet.sqlite"
    } else {
      ".local/share/ord/wallet.sqlite"
    })
    .exists());

  Test::with_tempdir(tempdir)
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

  assert!(tempdir
    .path()
    .join(if cfg!(target_os = "macos") {
      "Library/Application Support/ord/wallet.sqlite"
    } else {
      ".local/share/ord/wallet.sqlite"
    })
    .exists());

  Ok(())
}

#[test]
fn fund() -> Result {
  let tempdir = Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .set_home_to_tempdir()
    .output()?
    .tempdir;

  Test::with_tempdir(tempdir)
    .command("wallet fund")
    .set_home_to_tempdir()
    .expected_stdout("tb1qtprrd8eadw3kd4h44yplkh85mj3hv0qwunj99h\n")
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
