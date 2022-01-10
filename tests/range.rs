use super::*;

#[test]
fn genesis() -> Result {
  Test::new()?
    .args(&["range", "0"])
    .expected_stdout("0 5000000000\n")
    .run()
}

#[test]
fn second_block() -> Result {
  Test::new()?
    .args(&["range", "1"])
    .expected_stdout("5000000000 10000000000\n")
    .run()
}
