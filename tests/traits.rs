use super::*;

#[test]
fn invalid_ordinal() {
  Test::new()
    .args(&["traits", "2099999997690000"])
    .stderr_regex("error: Invalid value \"2099999997690000\" for '<ORDINAL>': Invalid ordinal\n.*")
    .expected_status(2)
    .run();
}

#[test]
fn valid_ordinal() {
  Test::new()
    .args(&["traits", "0"])
    .expected_stdout(
      "\
number: 0
decimal: 0.0
degree: 0°0′0″0‴
name: nvtdijuwxlp
height: 0
cycle: 0
epoch: 0
period: 0
offset: 0
rarity: mythic
",
    )
    .run();
}
