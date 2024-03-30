use {
  super::*,
  ord::subcommand::list::{Output, Range},
};

#[test]
fn output_found() {
  let core = mockcore::spawn();
  let output = CommandBuilder::new(
    "--index-sats list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
  )
  .core(&core)
  .run_and_deserialize_output::<Output>();

  assert_eq!(
    output,
    Output {
      ranges: Some(vec![Range {
        end: 50 * COIN_VALUE,
        name: "nvtdijuwxlp".into(),
        offset: 0,
        rarity: "mythic".parse().unwrap(),
        size: 50 * COIN_VALUE,
        start: 0,
      }]),
      spent: false,
    }
  );
}

#[test]
fn output_not_found() {
  let core = mockcore::spawn();
  CommandBuilder::new(
    "--index-sats list 0000000000000000000000000000000000000000000000000000000000000000:0",
  )
  .core(&core)
  .expected_exit_code(1)
  .expected_stderr("error: output not found\n")
  .run_and_extract_stdout();
}

#[test]
fn no_satoshi_index() {
  let core = mockcore::spawn();
  CommandBuilder::new("list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0")
    .core(&core)
    .expected_stderr("error: list requires index created with `--index-sats` flag\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
