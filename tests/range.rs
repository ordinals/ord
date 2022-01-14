use super::*;

#[test]
fn genesis() -> Result {
  Test::new()?
    .args(&["range", "0"])
    .expected_stdout("2099999997689999 2099994997689999\n")
    .run()
}

#[test]
fn second_block() -> Result {
  Test::new()?
    .args(&["range", "1"])
    .expected_stdout("2099994997689999 2099989997689999\n")
    .run()
}

#[test]
fn last_block_with_subsidy() -> Result {
  Test::new()?
    .args(&["range", "6929999"])
    .expected_stdout("0 -1\n")
    .run()
}

#[test]
fn first_block_without_subsidy() -> Result {
  Test::new()?
    .args(&["range", "6930000"])
    .expected_stdout("-1 -1\n")
    .run()
}
