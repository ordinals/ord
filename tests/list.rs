use super::*;

#[test]
fn output_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0")
    .rpc_server(&rpc_server)
    .expected_stdout("[0,5000000000)\n")
    .run();
}

#[test]
fn output_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("list 150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0")
    .rpc_server(&rpc_server)
    .expected_status(1)
    .run();
}

#[test]
#[ignore]
fn two_fee_paying_transaction_range() {
  SlowTest::new()
    .command("list 7f3b38a0bc60f581fd7f4b178ca2a697575000e212c8752b455ec134d160ea9a:0")
    .blocks(102)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 10,
      recipient: None,
    })
    .transaction(TransactionOptions {
      slots: &[(2, 0, 0)],
      output_count: 1,
      fee: 10,
      recipient: None,
    })
    .blocks(1)
    .expected_stdout(
      "[515000000000,520000000000)\n[9999999990,10000000000)\n[14999999990,15000000000)\n",
    )
    .run()
}

#[test]
#[ignore]
// no value in the output utxo
fn null_output() {
  SlowTest::new()
    .command("list 3dbc87de25bf5a52ddfa8038bda36e09622f4dec7951d81ac43e4b0e8c54bc5b:0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 50 * 100_000_000,
      recipient: None,
    })
    .blocks(1)
    .expected_stdout("")
    .run()
}

// use a no value utxo as an input
#[test]
#[ignore]
fn null_input() {
  SlowTest::new()
    .command("list 3dbc87de25bf5a52ddfa8038bda36e09622f4dec7951d81ac43e4b0e8c54bc5b:0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 50 * 100_000_000,
      recipient: None,
    })
    .blocks(1)
    .transaction(TransactionOptions {
      slots: &[(102, 1, 0)],
      output_count: 1,
      fee: 0,
      recipient: None,
    })
    .expected_stdout("")
    .run()
}
