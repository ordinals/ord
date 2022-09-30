use super::*;

#[test]
fn ok() {
  CommandBuilder::new("parse a")
    .expected_stdout("2099999997689999\n")
    .run();
}

#[test]
fn err() {
  CommandBuilder::new("parse A")
    .stderr_regex("error: .*: invalid digit found in string.*")
    .expected_exit_code(2)
    .run();
}
