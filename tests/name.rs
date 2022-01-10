use super::*;

#[test]
fn empty() -> Result {
  Test::new()?
    .args(&["name", ""])
    .expected_stdout("0\n")
    .run()
}

#[test]
fn a() -> Result {
  Test::new()?
    .args(&["name", "a"])
    .expected_stdout("1\n")
    .run()
}

#[test]
fn b() -> Result {
  Test::new()?
    .args(&["name", "b"])
    .expected_stdout("2\n")
    .run()
}
