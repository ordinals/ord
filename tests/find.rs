use super::*;

#[test]
fn first_satoshi() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run()
}

#[test]
fn first_satoshi_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 0 --slot")
    .expected_stdout("0.0.0.0\n")
    .run()
}

#[test]
fn second_satoshi() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 1 --as-of-height 0")
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:1\n")
    .run()
}

#[test]
fn second_satoshi_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 1 --as-of-height 0 --slot")
    .expected_stdout("0.0.0.1\n")
    .run()
}

#[test]
fn first_satoshi_of_second_block() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 5000000000 --as-of-height 1")
    .expected_stdout("e5fb252959bdc7727c80296dbc53e1583121503bb2e266a609ebc49cf2a74c1d:0:0\n")
    .run()
}

#[test]
fn first_satoshi_of_second_block_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 5000000000 --as-of-height 1 --slot")
    .expected_stdout("1.0.0.0\n")
    .run()
}

#[test]
fn first_satoshi_spent_in_second_block() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 1")
    .expected_stdout("1e8149c3be0dd66b1cbcb4652d15bea04a9bc8d515c4f544e71bb35a9cba1ed0:0:0\n")
    .run()
}

#[test]
fn first_satoshi_spent_in_second_block_slot() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 1 --slot")
    .expected_stdout("1.1.0.0\n")
    .run()
}
