use super::*;

#[test]
fn genesis() -> Result {
  Test::new()?
    .args(&["range", "0"])
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_block() -> Result {
  Test::new()?
    .args(&["range", "1"])
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn last_block_with_subsidy() -> Result {
  Test::new()?
    .args(&["range", "6929999"])
    .expected_stdout("[2099999997689999,2099999997690000)\n")
    .run()
}

#[test]
fn first_block_without_subsidy() -> Result {
  Test::new()?
    .args(&["range", "6930000"])
    .expected_stdout("[2099999997690000,2099999997690000)\n")
    .run()
}

#[test]
fn genesis_names() -> Result {
  Test::new()?
    .args(&["range", "--name", "0"])
    .expected_stdout("[nvtdijuwxlo,nvtcsezkbtg)\n")
    .run()
}

#[test]
fn names_before_last() -> Result {
  Test::new()?
    .args(&["range", "--name", "6929998"])
    .expected_stdout("[a,)\n")
    .run()
}
