use super::*;

#[test]
#[cfg(target_os = "macos")]
fn init() -> Result {
  Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .expected_path("Library/Application Support/ord/wallet.sqlite")
    .run()
}

#[test]
#[cfg(target_os = "linux")]
fn init() -> Result {
  Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .expected_path(".local/share/ord/wallet.sqlite")
    .run()
}

#[test]
fn fund() -> Result {
  Test::new()?
    .command("wallet fund")
    .set_home_to_tempdir()
    .expected_stdout("tb1qtprrd8eadw3kd4h44yplkh85mj3hv0qwunj99h\n")
    .run()
}
