use {super::*, ord::subcommand::list::Output};

#[test]
fn output_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let output = CommandBuilder::new(
    "--index-sats list 97ddfbbae6be97fd6cdf3e7ca13232a3afff2353e29badfab7f73011edd4ced9:0",
  )
  .rpc_server(&rpc_server)
  .output::<Vec<Output>>();

  assert_eq!(
    output,
    vec![Output {
      output: "97ddfbbae6be97fd6cdf3e7ca13232a3afff2353e29badfab7f73011edd4ced9:0"
        .parse()
        .unwrap(),
      start: 0,
      size: 50 * COIN_VALUE,
      rarity: "mythic".parse().unwrap(),
      name: "bgmbqkqiqsxl".into(),
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
  CommandBuilder::new("list 97ddfbbae6be97fd6cdf3e7ca13232a3afff2353e29badfab7f73011edd4ced9:0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: list requires index created with `--index-sats` flag\n")
    .expected_exit_code(1)
    .run();
}
