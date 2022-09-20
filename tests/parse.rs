use super::*;

#[test]
fn ok() {
  Test::new()
    .args(&["parse", "a"])
    .expected_stdout("2099999997689999\n")
    .run()
}

#[test]
fn err() {
  Test::new()
    .args(&["parse", ""])
    .expected_stderr("error: cannot parse integer from empty string\n")
    .expected_status(1)
    .run()
}
