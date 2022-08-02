use super::*;

#[test]
fn genesis() {
  Test::new()
    .args(&["range", "0"])
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_block() {
  Test::new()
    .args(&["range", "1"])
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn last_block_with_subsidy() {
  Test::new()
    .args(&["range", "6929999"])
    .expected_stdout("[2099999997689999,2099999997690000)\n")
    .run()
}

#[test]
fn first_block_without_subsidy() {
  Test::new()
    .args(&["range", "6930000"])
    .expected_stdout("[2099999997690000,2099999997690000)\n")
    .run()
}

#[test]
fn genesis_names() {
  Test::new()
    .args(&["range", "--name", "0"])
    .expected_stdout("[nvtdijuwxlp,nvtcsezkbth)\n")
    .run()
}

#[test]
fn names_before_last() {
  Test::new()
    .args(&["range", "--name", "6929998"])
    .expected_stdout("[b,a)\n")
    .run()
}

#[test]
fn last_name() {
  Test::new()
    .args(&["range", "--name", "6929999"])
    .expected_stdout("[a,)\n")
    .run()
}

#[test]
fn block_with_no_subsidy_range() {
  Test::new()
    .args(&["range", "--name", "6930000"])
    .expected_stdout("[,)\n")
    .run()
}
