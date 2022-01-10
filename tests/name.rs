use super::*;

#[test]
fn name() -> Result {
  Test::new()?
    .args(&["name", "a"])
    .expected_stdout("1\n")
    .run()
}
