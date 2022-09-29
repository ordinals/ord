use super::*;

#[test]
fn basic() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("info")
    .rpc_server(&rpc_server)
    .stdout_regex(
      r"
        blocks indexed: 1
        utxos indexed: 1
        outputs traversed: 1
        tree height: \d+
        free pages: \d+
        stored: .*
        overhead: .*
        fragmented: .*
      "
      .unindent(),
    )
    .run();
}
