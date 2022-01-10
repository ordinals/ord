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

#[test]
fn end_of_range() -> Result {
  Test::new()?
    .args(&["name", "nvtdijuwxlo"])
    .expected_stdout("2099999997689999\n")
    .run()
}

#[test]
fn out_of_range() -> Result {
  Test::new()?
    .args(&["name", "nvtdijuwxlp"])
    .expected_stderr("error: Name out of range\n")
    .expected_status(1)
    .run()
}

#[test]
fn invalid() -> Result {
  Test::new()?
    .args(&["name", "0"])
    .expected_stderr("error: Invalid name\n")
    .expected_status(1)
    .run()
}
