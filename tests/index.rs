use super::*;

#[test]
fn custom_index_size() {
  let state = Test::new()
    .command("--max-index-size 1mib find 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .blocks(1)
    .output()
    .state;

  assert_eq!(
    state
      .ord_data_dir()
      .join("index.redb")
      .metadata()
      .unwrap()
      .len(),
    1 << 20
  );
}

#[test]
fn height_limit() {
  Test::new()
    .command("find 5000000000")
    .expected_stdout("150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0:0\n")
    .blocks(1)
    .run();

  Test::new()
    .command("--height-limit 0 find 5000000000")
    .expected_stderr("error: Ordinal has not been mined as of index height\n")
    .expected_status(1)
    .blocks(1)
    .run();
}
