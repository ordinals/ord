use super::*;

#[test]
fn first_satoshi() -> Result {
  Test::new()?
    .command("find 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run()
}

#[test]
#[ignore]
fn first_satoshi_slot() -> Result {
  Test::new()?
    .command("find 0 --slot")
    .expected_stdout("0x0x0x0\n")
    .block()
    .run()
}

#[test]
fn second_satoshi() -> Result {
  Test::new()?
    .command("find 1")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:1\n")
    .run()
}

#[test]
#[ignore]
fn second_satoshi_slot() -> Result {
  Test::new()?
    .command("find 1 --slot")
    .expected_stdout("0x0x0x1\n")
    .block()
    .run()
}

#[test]
fn first_satoshi_of_second_block() -> Result {
  Test::new()?
    .command("find 5000000000")
    .expected_stdout("104a8ee40f039ba83ceda9de4c6eb7d8587704168f687b315974d307c93b9caf:0:0\n")
    .block()
    .run()
}

#[test]
#[ignore]
fn first_satoshi_of_second_block_slot() -> Result {
  Test::new()?
    .command("find 5000000000 --slot")
    .expected_stdout("1x0x0x0\n")
    .block()
    .block()
    .run()
}

#[test]
fn first_satoshi_spent_in_second_block() -> Result {
  Test::new()?
    .command("find 0")
    .expected_stdout("d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0:0:0\n")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .run()
}

#[test]
#[ignore]
fn first_satoshi_spent_in_second_block_slot() -> Result {
  Test::new()?
    .command("find 0 --slot")
    .expected_stdout("1x1x0x0\n")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .run()
}

#[test]
#[ignore]
fn regression_empty_block_crash() -> Result {
  Test::new()?
    .command("find 0 --slot")
    .block()
    .block_with_coinbase(CoinbaseOptions {
      include_coinbase_transaction: false,
      ..Default::default()
    })
    .expected_stdout("0x0x0x0\n")
    .run()
}

#[test]
#[ignore]
fn mining_and_spending_transaction_in_same_block() -> Result {
  Test::new()?
    .command("find 0 --slot")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .transaction(TransactionOptions {
      slots: &[(1, 1, 0)],
      output_count: 1,
      fee: 0,
    })
    .expected_stdout("1x2x0x0\n")
    .run()
}

#[test]
fn empty_index() -> Result {
  Test::new()?
    .expected_stderr("error: Ordinal has not been mined as of index height\n")
    .expected_status(1)
    .command("find 0")
    .run()
}

#[test]
fn unmined_satoshi_in_second_block() -> Result {
  Test::new()?
    .block()
    .expected_stderr("error: Ordinal has not been mined as of index height\n")
    .expected_status(1)
    .command("find 5000000000")
    .run()
}

#[test]
fn unmined_satoshi_in_first_block() -> Result {
  Test::new()?
    .expected_stderr("error: Ordinal has not been mined as of index height\n")
    .expected_status(1)
    .command("find 0")
    .run()
}
