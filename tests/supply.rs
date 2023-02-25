use {super::*, ord::subcommand::supply::Output};

#[test]
fn genesis() {
  assert_eq!(
    CommandBuilder::new("supply").output::<Output>(),
    Output {
      supply: 8399999990760000,
      first: 0,
      last: 8399999990759999,
      last_mined_in_block: 27719999
    }
  );
}
