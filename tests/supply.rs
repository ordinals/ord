use {super::*, ord::subcommand::supply::Output};

#[test]
fn genesis() {
  assert_eq!(
    CommandBuilder::new("supply").run_and_check_output::<Output>(),
    Output {
      supply: 2099999997690000,
      first: 0,
      last: 2099999997689999,
      last_mined_in_block: 6929999
    }
  );
}
