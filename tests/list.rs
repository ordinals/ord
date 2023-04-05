use {super::*, ord::subcommand::list::Output};

#[test]
fn output_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let output = CommandBuilder::new(
    "--index-sats list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
  )
  .rpc_server(&rpc_server)
  .output::<Vec<Output>>();

  assert_eq!(
    output,
    vec![Output {
      output: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
        .parse()
        .unwrap(),
      start: 0,
      end: 50 * COIN_VALUE,
      size: 50 * COIN_VALUE,
      offset: 0,
      rarity: "mythic".parse().unwrap(),
      name: "nvtdijuwxlp".into(),
    }]
  );
}

#[test]
fn output_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new(
    "--index-sats list 0000000000000000000000000000000000000000000000000000000000000000:0",
  )
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: output not found\n")
  .run();
}

#[test]
fn no_satoshi_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: list requires index created with `--index-sats` flag\n")
    .expected_exit_code(1)
    .run();
}
