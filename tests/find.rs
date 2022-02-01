use super::*;

#[test]
fn first_satoshi() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .block()
    .run()
}

#[test]
fn first_satoshi_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 0 --slot")
    .expected_stdout("0.0.0.0\n")
    .block()
    .run()
}

#[test]
fn second_satoshi() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 1 --as-of-height 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:1\n")
    .block()
    .run()
}

#[test]
fn second_satoshi_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 1 --as-of-height 0 --slot")
    .expected_stdout("0.0.0.1\n")
    .block()
    .run()
}

#[test]
fn first_satoshi_of_second_block() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 5000000000 --as-of-height 1")
    .expected_stdout("9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0:0\n")
    .block()
    .block()
    .run()
}

#[test]
fn first_satoshi_of_second_block_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 5000000000 --as-of-height 1 --slot")
    .expected_stdout("1.0.0.0\n")
    .block()
    .block()
    .run()
}

#[test]
fn first_satoshi_spent_in_second_block() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 1")
    .expected_stdout("72e60639a1dcc6263ed214a1db0dc9545bf65d9327e5a60e84bd3db7fbb4c2fa:0:0\n")
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 1)
    .run()
}

#[test]
fn first_satoshi_spent_in_second_block_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 1 --slot")
    .expected_stdout("1.1.0.0\n")
    .block()
    .block()
    .transaction(&[(0, 0, 0)], 1)
    .run()
}

#[test]
fn regression_empty_block_crash() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --slot --as-of-height 1")
    .block()
    .block_without_coinbase()
    .expected_stdout("0.0.0.0\n")
    .run()
}
