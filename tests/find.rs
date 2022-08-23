use super::*;

#[test]
fn first_satoshi() {
  Test::new()
    .command("find 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run();
}

#[test]
#[ignore]
fn first_satoshi_slot() {
  Test::new()
    .command("find 0 --slot")
    .expected_stdout("0x0x0x0\n")
    .blocks(1)
    .run();
}

#[test]
fn second_satoshi() {
  Test::new()
    .command("find 1")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:1\n")
    .run();
}

#[test]
#[ignore]
fn second_satoshi_slot() {
  Test::new()
    .command("find 1 --slot")
    .expected_stdout("0x0x0x1\n")
    .blocks(1)
    .run()
}

#[test]
fn first_satoshi_of_second_block() {
  Test::new()
    .command("find 5000000000")
    .blocks(1)
    .expected_stdout("150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0:0\n")
    .run();
}

#[test]
#[ignore]
fn first_satoshi_of_second_block_slot() {
  Test::new()
    .command("find 5000000000 --slot")
    .expected_stdout("1x0x0x0\n")
    .blocks(1)
    .blocks(1)
    .run();
}

#[test]
fn first_satoshi_spent_in_second_block() {
  Test::new()
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

#[test]
#[ignore]
fn first_satoshi_spent_in_second_block_slot() {
  Test::new()
    .command("find 0 --slot")
    .expected_stdout("1x1x0x0\n")
    .blocks(1)
    .blocks(1)
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
      recipient: None,
    })
    .run();
}

#[test]
#[ignore]
fn mining_and_spending_transaction_in_same_block() {
  Test::new()
    .command("find 0 --slot")
    .blocks(1)
    .blocks(1)
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
      recipient: None,
    })
    .transaction(TransactionOptions {
      slots: &[(1, 1, 0)],
      output_count: 1,
      fee: 0,
      recipient: None,
    })
    .expected_stdout("1x2x0x0\n")
    .run();
}

#[test]
fn unmined_satoshi_in_second_block() {
  Test::new()
    .expected_stderr("error: Ordinal has not been mined as of index height\n")
    .expected_status(1)
    .command("find 5000000000")
    .run();
}
