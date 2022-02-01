use super::*;

#[test]
fn first_satoshi() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 0")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
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
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:1\n")
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
    .expected_stdout("d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0:0:0\n")
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
    .block_with_coinbase(false)
    .expected_stdout("0.0.0.0\n")
    .run()
}

#[test]
fn index_multiple_blockfiles() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 1 --slot")
    .expected_stdout("1.1.0.0\n")
    .block()
    .blockfile()
    .block()
    .transaction(&[(0, 0, 0)], 1)
    .run()
}
