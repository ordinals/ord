use super::*;

#[test]
fn init() -> Result {
  let test = Test::new()?;

  test
    .command("wallet init")
    .set_home_to_tempdir()
    .expected_status(0)
    .expected_stderr("Wallet initialized.\n")
    .expected_path("Library/Application Support/ord/wallet.sqlite")
    .run()
}

#[test]
fn fund() -> Result {
  let test = Test::new()?;

  test
    .command("wallet fund")
    .set_home_to_tempdir()
    .expected_stdout("tb1qtprrd8eadw3kd4h44yplkh85mj3hv0qwunj99h\n")
    .run()
}
