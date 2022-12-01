use super::*;

#[test]
fn genesis() {
  CommandBuilder::new("subsidy 0")
    .expected_stdout("0\t5000000000\tnvtdijuwxlp\n")
    .run();
}

#[test]
fn second_block() {
  CommandBuilder::new("subsidy 1")
    .expected_stdout("5000000000\t5000000000\tnvtcsezkbth\n")
    .run();
}

#[test]
fn second_to_last_block_with_subsidy() {
  CommandBuilder::new("subsidy 6929998")
    .expected_stdout("2099999997689998\t1\tb\n")
    .run();
}

#[test]
fn last_block_with_subsidy() {
  CommandBuilder::new("subsidy 6929999")
    .expected_stdout("2099999997689999\t1\ta\n")
    .run();
}

#[test]
fn first_block_without_subsidy() {
  CommandBuilder::new("subsidy 6930000")
    .expected_stderr("error: block 6930000 has no subsidy\n")
    .expected_exit_code(1)
    .run();
}
