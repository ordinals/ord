use super::*;

#[test]
fn genesis() {
  Test::new()
    .args(&["supply"])
    .expected_stdout(
      &"
      supply: 2099999997690000
      first: 0
      last: 2099999997689999
      last mined in block: 6929999
      "
      .unindent(),
    )
    .run();
}
