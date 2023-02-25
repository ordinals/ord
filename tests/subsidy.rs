use {super::*, ord::subcommand::subsidy::Output};

#[test]
fn genesis() {
  assert_eq!(
    CommandBuilder::new("subsidy 0").output::<Output>(),
    Output {
      first: 0,
      subsidy: 5000000000,
      name: "bgmbqkqiqsxl".into(),
    }
  );
}

#[test]
fn second_block() {
  assert_eq!(
    CommandBuilder::new("subsidy 1").output::<Output>(),
    Output {
      first: 5000000000,
      subsidy: 5000000000,
      name: "bgmbpulndxfd".into(),
    }
  );
}

#[test]
fn second_to_last_block_with_subsidy() {
  assert_eq!(
    CommandBuilder::new("subsidy 27719998").output::<Output>(),
    Output {
      first: 8399999990759998,
      subsidy: 1,
      name: "b".into(),
    }
  );
}

#[test]
fn last_block_with_subsidy() {
  assert_eq!(
    CommandBuilder::new("subsidy 27719999").output::<Output>(),
    Output {
      first: 8399999990759999,
      subsidy: 1,
      name: "a".into(),
    }
  );
}

#[test]
fn first_block_without_subsidy() {
  CommandBuilder::new("subsidy 27720000")
    .expected_stderr("error: block 27720000 has no subsidy\n")
    .expected_exit_code(1)
    .run();
}
