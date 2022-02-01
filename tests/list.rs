use super::*;

#[test]
fn first_coinbase_transaction() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks 0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
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
      "list --blocksdir blocks a3f7b03f71988d4f91fea260405dbf3f3586eb134ad01dad15de63053e4985d0:0",
    )
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 2)
    .expected_stdout("[0,2500000000)\n")
    .run()?;

  Test::new()?
    .command(
      "list --blocksdir blocks a3f7b03f71988d4f91fea260405dbf3f3586eb134ad01dad15de63053e4985d0:1",
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
      "list --blocksdir blocks db7d0407c1548d2ceb00fd37447dfe723b954cc69cd5cbfd6b020f667db807a2:0",
    )
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 2)
    .block()
    .transaction(&[(1, 1, 0), (1, 1, 1)], 1)
    .expected_stdout("[0,2500000000)\n[2500000000,5000000000)\n")
    .run()
}
