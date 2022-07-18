use super::*;

#[test]
fn fund() -> Result {
  Test::new()?
    .command("wallet fund")
    .expected_stdout("bcrt1qtprrd8eadw3kd4h44yplkh85mj3hv0qw76tgj7\n")
    .run()
}
