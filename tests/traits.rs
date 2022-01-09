use super::*;

#[test]
fn zero() -> Result {
  Test::new()?
    .args(&["traits", "0"])
    .expected_stdout("divine\n")
    .run()
}
