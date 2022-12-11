use super::*;

#[test]
fn find_command_returns_satpoint_for_ordinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("--index-satoshis find 0")
    .rpc_server(&rpc_server)
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run();
}

#[test]
fn unmined_ordinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("--index-satoshis find 5000000000")
    .rpc_server(&rpc_server)
    .expected_stderr("error: ordinal has not been mined as of index height\n")
    .expected_exit_code(1)
    .run();
}

#[test]
fn no_satoshi_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("find 0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: find requires index created with `--index-satoshis` flag\n")
    .expected_exit_code(1)
    .run();
}
