use super::*;

#[test]
fn first_coinbase_transaction() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
    )
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_coinbase_transaction() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks e5fb252959bdc7727c80296dbc53e1583121503bb2e266a609ebc49cf2a74c1d:0",
    )
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}
