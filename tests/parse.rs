use super::*;

#[test]
fn ok() {
  TestCommand::new()
    .command("parse a")
    .expected_stdout("2099999997689999\n")
    .run();
}

#[test]
fn err() {
  TestCommand::new()
    .command("parse A")
    .expected_stderr("error: invalid digit found in string\n")
    .expected_status(1)
    .run();
}
