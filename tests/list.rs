use super::*;

#[test]
fn first_coinbase_transaction() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
    )
    .block()
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_coinbase_transaction() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks 9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0",
    )
    .block()
    .block()
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn third_coinbase_transaction_is_not_duplicate() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks 8aa5103b13b5b233ac417ee31f21820c9284af2b7a2080a142c2d20e1697b0f4:0",
    )
    .block()
    .block()
    .block()
    .expected_stdout("[10000000000,15000000000)\n")
    .run()
}

#[test]
fn split_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks 7ab338c0e46c95c119ba8ff0a3f6cf64f56f35ba5553a399fdd38169cca00be3:0",
    )
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 2)
    .expected_stdout("[0,2500000000)\n")
    .run()?;

  Test::new()?
    .command(
      "list --blocksdir blocks 7ab338c0e46c95c119ba8ff0a3f6cf64f56f35ba5553a399fdd38169cca00be3:1",
    )
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 2)
    .expected_stdout("[2500000000,5000000000)\n")
    .run()
}

#[test]
fn merge_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks fe283c08e46269a7bbe36b629bc2be55d604152419818e94330477c9c3487eec:0",
    )
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 2)
    .block()
    .transaction(&[(1, 1, 0), (1, 1, 1)], 1)
    .expected_stdout("[0,2500000000)\n[2500000000,5000000000)\n")
    .run()
}
