use super::*;

#[test]
fn genesis() {
  SlowTest::new()
    .args(&["range", "0"])
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_block() {
  SlowTest::new()
    .args(&["range", "1"])
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn last_block_with_subsidy() {
  SlowTest::new()
    .args(&["range", "6929999"])
    .expected_stdout("[2099999997689999,2099999997690000)\n")
    .run()
}

#[test]
fn first_block_without_subsidy() {
  SlowTest::new()
    .args(&["range", "6930000"])
    .expected_stdout("[2099999997690000,2099999997690000)\n")
    .run()
}

#[test]
fn genesis_names() {
  SlowTest::new()
    .args(&["range", "--name", "0"])
    .expected_stdout("[nvtdijuwxlp,nvtcsezkbth)\n")
    .run()
}

#[test]
fn names_before_last() {
  SlowTest::new()
    .args(&["range", "--name", "6929998"])
    .expected_stdout("[b,a)\n")
    .run()
}

#[test]
fn last_name() {
  SlowTest::new()
    .args(&["range", "--name", "6929999"])
    .expected_stdout("[a,)\n")
    .run()
}

#[test]
fn block_with_no_subsidy_range() {
  SlowTest::new()
    .args(&["range", "--name", "6930000"])
    .expected_stdout("[,)\n")
    .run()
}
