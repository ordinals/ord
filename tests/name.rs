use super::*;

#[test]
fn empty() {
  Test::new()
    .args(&["name", ""])
    .expected_stderr("error: Invalid name\n")
    .expected_status(1)
    .run()
}

#[test]
fn a() {
  Test::new()
    .args(&["name", "a"])
    .expected_stdout("2099999997689999\n")
    .run()
}

#[test]
fn b() {
  Test::new()
    .args(&["name", "b"])
    .expected_stdout("2099999997689998\n")
    .run()
}

#[test]
fn end_of_range() {
  Test::new()
    .args(&["name", "nvtdijuwxlp"])
    .expected_stdout("0\n")
    .run()
}

#[test]
fn out_of_range() {
  Test::new()
    .args(&["name", "nvtdijuwxlr"])
    .expected_stderr("error: Name out of range\n")
    .expected_status(1)
    .run()
}

#[test]
fn invalid() {
  Test::new()
    .args(&["name", "0"])
    .expected_stderr("error: Invalid name\n")
    .expected_status(1)
    .run()
}
