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
      address: None,
      indexed: true,
      inscriptions: Some(Vec::new()),
      runes: None,
      sat_ranges: Some(vec![Range {
        end: 50 * COIN_VALUE,
        name: "nvtdijuwxlp".into(),
        offset: 0,
        rarity: "mythic".parse().unwrap(),
        size: 50 * COIN_VALUE,
        start: 0,
       }]),
      script_pubkey: "OP_PUSHBYTES_65 04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f OP_CHECKSIG".to_string(),
      spent: false,
      transaction: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b".to_string(),
      value: 5000000000,
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
