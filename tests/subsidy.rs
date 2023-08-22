use {super::*, ord::subcommand::subsidy::Output};

#[test]
fn genesis() {
  assert_eq!(
    CommandBuilder::new("subsidy 0").run_and_check_output::<Output>(),
    Output {
      first: 0,
      subsidy: 5000000000,
      name: "nvtdijuwxlp".into(),
    }
  );
}

#[test]
fn second_block() {
  assert_eq!(
    CommandBuilder::new("subsidy 1").run_and_check_output::<Output>(),
    Output {
      first: 5000000000,
      subsidy: 5000000000,
      name: "nvtcsezkbth".into(),
    }
  );
}

#[test]
fn second_to_last_block_with_subsidy() {
  assert_eq!(
    CommandBuilder::new("subsidy 6929998").run_and_check_output::<Output>(),
    Output {
      first: 2099999997689998,
      subsidy: 1,
      name: "b".into(),
    }
  );
}

#[test]
fn last_block_with_subsidy() {
  assert_eq!(
    CommandBuilder::new("subsidy 6929999").run_and_check_output::<Output>(),
    Output {
      first: 2099999997689999,
      subsidy: 1,
      name: "a".into(),
    }
  );
}

#[test]
fn first_block_without_subsidy() {
  CommandBuilder::new("subsidy 6930000")
    .expected_stderr("error: block 6930000 has no subsidy\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
