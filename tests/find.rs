use super::*;

#[test]
fn find_command_returns_satpoint_for_ordinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("find 0")
    .rpc_server(&rpc_server)
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run();
}

#[test]
fn unmined_ordinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("find 5000000000")
    .rpc_server(&rpc_server)
    .expected_stderr("error: Ordinal has not been mined as of index height\n")
    .expected_status(1)
    .run();
}

#[test]
fn first_satoshi_spent_in_second_block() {
  SlowTest::new()
    .command("find 0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 0,
      recipient: None,
    })
    .blocks(1)
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run();
}
