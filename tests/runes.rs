use {super::*, ord::subcommand::runes::Output};

// todo:
// - test with non-empty output
// - test that flag is required

#[test]
fn empty() {
  assert_eq!(
    CommandBuilder::new("--index-runes-pre-alpha-i-agree-to-get-rekt --regtest runes")
      .run_and_deserialize_output::<Output>(),
    Output {
      runes: BTreeMap::new(),
    }
  );
}
