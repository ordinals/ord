use super::*;

#[test]
fn genesis() -> Result {
  Test::new()?
    .args(&["supply"])
    .expected_stdout(
      &"
      supply: 2099999997690000
      first: 2099999997689999
      last subsidy block: 6929999
      "
      .unindent(),
    )
    .run()
}
