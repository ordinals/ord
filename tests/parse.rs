use super::*;

#[test]
fn ok() {
  TestCommand::new("parse a")
    .expected_stdout("2099999997689999\n")
    .run();
}

#[test]
fn err() {
  TestCommand::new("parse A")
    .stderr_regex("error: .*: invalid digit found in string.*")
    .expected_status(2)
    .run();
}
