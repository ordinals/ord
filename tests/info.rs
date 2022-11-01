use super::*;

#[test]
fn basic() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("info")
    .rpc_server(&rpc_server)
    .stdout_regex(
      "
        blocks indexed\t1
        utxos indexed\t1
        outputs traversed\t1
        ordinal ranges\t1
        tree height\t\\d+
        free pages\t\\d+
        stored\t.*
        overhead\t.*
        fragmented\t.*
      "
      .unindent(),
    )
    .run();
}
