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
      "list --blocksdir blocks e5fb252959bdc7727c80296dbc53e1583121503bb2e266a609ebc49cf2a74c1d:0",
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
      "list --blocksdir blocks a224a289ba1d28e50f7b636bbc9d8939e06ad4b884c98270bd07402edcbaf5b6:0",
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
      "list --blocksdir blocks bba261e0b1195f850136e761c64e4ba75a8a60bc866da5e1b68299af967f332e:0",
    )
    .block()
    .block()
    .transaction((0, 0, 0), 2)
    .expected_stdout("[0,2500000000)\n")
    .run()?;

  Test::new()?
    .command(
      "list --blocksdir blocks bba261e0b1195f850136e761c64e4ba75a8a60bc866da5e1b68299af967f332e:1",
    )
    .block()
    .block()
    .transaction((0, 0, 0), 2)
    .expected_stdout("[2500000000,5000000000)\n")
    .run()
}

#[test]
fn merge_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command(
      "list --blocksdir blocks da9ebceb6c8190b64bf69037def994cf41fc871bf824e6c0392167387bc464e5:0",
    )
    .block()
    .block()
    .transaction((0, 0, 0), 2)
    .block()
    .transaction_2(&[(1, 1, 0), (1, 1, 1)], 1)
    .expected_stdout("[0,2500000000)\n[2500000000,5000000000)\n")
    .run()
}
