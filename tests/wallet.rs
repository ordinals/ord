use super::*;

#[test]
fn init() -> Result {
  Test::new()?
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .expected_path(if cfg!(target_os = "macos") {
      "Library/Application Support/ord/wallet.sqlite"
    } else {
      ".local/share/ord/wallet.sqlite"
    })
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
