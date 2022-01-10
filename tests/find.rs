use super::*;

#[test]
fn first_satoshi() -> Result {
  Test::new()?
    .args(&["find", "--blocksdir", "blocks", "0", "0"])
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0\n")
    .run()
}

#[test]
fn first_satoshi_of_second_block() -> Result {
  Test::new()?
    .args(&["find", "--blocksdir", "blocks", "5000000000", "1"])
    .expected_stdout("e5fb252959bdc7727c80296dbc53e1583121503bb2e266a609ebc49cf2a74c1d:0\n")
    .run()
}
